/*
 * ‌
 * Hedera Rust SDK
 * ​
 * Copyright (C) 2023 - 2023 Hedera Hashgraph, LLC
 * ​
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 * ‍
 */

use std::borrow::Cow;
use std::ops::Range;

use hedera_proto::services;
use once_cell::sync::OnceCell;
use prost::Message;

use crate::protobuf::FromProtobuf;
use crate::signer::AnySigner;
use crate::{
    AccountId,
    Error,
    TransactionHash,
    TransactionId,
};

pub(crate) struct SourceChunk<'a> {
    map: &'a TransactionSources,
    index: usize,
}

impl<'a> SourceChunk<'a> {
    fn range(&self) -> Range<usize> {
        self.map.chunks[self.index].clone()
    }

    pub(crate) fn transaction_id(&self) -> TransactionId {
        self.map.transaction_ids[self.index]
    }

    pub(crate) fn transactions(&self) -> &'a [services::Transaction] {
        &self.map.transactions()[self.range()]
    }

    pub(crate) fn signed_transactions(&self) -> &'a [services::SignedTransaction] {
        &self.map.signed_transactions[self.range()]
    }

    /// Returns The node account IDs for this chunk.
    ///
    /// Note: Every chunk has the same node account IDs.
    pub(crate) fn node_ids(&self) -> &'a [AccountId] {
        &self.map.node_ids
    }

    pub(crate) fn transaction_hashes(&self) -> &'a [TransactionHash] {
        &self.map.transaction_hashes()[self.range()]
    }
}

#[derive(Default, Clone)]
pub struct TransactionSources {
    signed_transactions: Box<[services::SignedTransaction]>,

    transactions: OnceCell<Vec<services::Transaction>>,

    /// offset of each chunk into `transactions`/`signed_transactions`
    chunks: Vec<Range<usize>>,

    /// Ordered list of transaction IDs (1 per chunk)
    transaction_ids: Vec<TransactionId>,

    /// Ordered list of node account IDs (all per chunk, same ordering)
    node_ids: Vec<AccountId>,

    transaction_hashes: OnceCell<Vec<TransactionHash>>,
}

impl TransactionSources {
    pub(crate) fn new(transactions: Vec<services::Transaction>) -> crate::Result<Self> {
        if transactions.is_empty() {
            return Err(Error::from_protobuf("`TransactionList` had no transactions"));
        }

        let signed_transactions: Result<Vec<_>, _> = transactions
            .iter()
            .map(|transaction| {
                if !transaction.signed_transaction_bytes.is_empty() {
                    let tx =
                        services::SignedTransaction::decode(&*transaction.signed_transaction_bytes)
                            .map_err(Error::from_protobuf)?;

                    return Ok(tx);
                }

                Err(Error::from_protobuf("Transaction had no signed transaction bytes"))
            })
            .collect();

        let signed_transactions = signed_transactions?;

        // ensure all signers (if any) are consistent for all signed transactions.
        // this doesn't compare or validate the signatures,
        // instead it ensures that all signatures in the first signed transation exist in *all* transactions and none extra exist.
        {
            let mut iter = signed_transactions.iter().map(|it| {
                it.sig_map
                    .as_ref()
                    .map(|it| {
                        let mut tmp = it
                            .sig_pair
                            .iter()
                            .map(|it| it.pub_key_prefix.as_slice())
                            .collect::<Vec<_>>();

                        // sort to be generous about signature ordering.
                        tmp.sort();

                        tmp
                    })
                    .unwrap_or_default()
            });

            // this should always be `Some`, buuuut, we lose nothing by doing it this way.
            if let Some(first) = iter.next() {
                if iter.any(|sigs| first != sigs.as_slice()) {
                    return Err(Error::from_protobuf("Transaction has mismatched signatures"));
                }
            }
        }

        let transaction_info: Result<Vec<_>, _> = signed_transactions
            .iter()
            .map(|it| {
                services::TransactionBody::decode(it.body_bytes.as_slice())
                    .map_err(Error::from_protobuf)
                    .and_then(|body| {
                        Ok((
                            TransactionId::from_protobuf(pb_getf!(body, transaction_id)?)?,
                            AccountId::from_protobuf(pb_getf!(body, node_account_id)?)?,
                        ))
                    })
            })
            .collect();

        let transaction_info = transaction_info?;

        let (chunks, transaction_ids, node_ids) = {
            let mut current: Option<&TransactionId> = None;

            let chunk_starts =
                transaction_info.iter().enumerate().filter_map(move |(index, (id, _))| {
                    if current != Some(id) {
                        current = Some(id);

                        return Some(index);
                    }

                    None
                });

            let mut chunks = Vec::new();

            let mut previous_start = None;

            // the start of one chunk is the end of the previous one.
            for end in chunk_starts {
                let start = previous_start.replace(end);

                if let Some(start) = start {
                    chunks.push(start..end);
                }
            }

            if let Some(start) = previous_start {
                chunks.push(start..transaction_info.len());
            }

            let chunks = chunks;

            let mut transaction_ids = Vec::with_capacity(chunks.len());
            let mut node_ids: Vec<_> = Vec::new();

            for chunk in &chunks {
                if transaction_ids.contains(&transaction_info[chunk.start].0) {
                    return Err(Error::from_protobuf(
                        "duplicate transaction ID between chunked transaction chunks",
                    ));
                }

                transaction_ids.push(transaction_info[chunk.start].0);

                // else ifs acting on different kinds of conditions are
                // personally more confusing than having the extra layer of nesting.
                #[allow(clippy::collapsible_else_if)]
                if node_ids.is_empty() {
                    node_ids = transaction_info[chunk.clone()].iter().map(|it| it.1).collect();
                } else {
                    if node_ids.iter().ne(transaction_info[chunk.clone()].iter().map(|it| &it.1)) {
                        return Err(Error::from_protobuf(
                            "TransactionList has inconsistent node account IDs",
                        ));
                    }
                }
            }

            (chunks, transaction_ids, node_ids)
        };

        Ok(Self {
            signed_transactions: signed_transactions.into_boxed_slice(),
            transactions: OnceCell::with_value(transactions),
            chunks,
            transaction_ids,
            node_ids,
            transaction_hashes: OnceCell::new(),
        })
    }

    pub(crate) fn sign_with(&self, signers: &[AnySigner]) -> Cow<'_, Self> {
        if signers.is_empty() {
            return Cow::Borrowed(self);
        }

        let mut signed_transactions = Cow::Borrowed(&self.signed_transactions);

        for signer in signers {
            let pk = signer.public_key().to_bytes_raw();

            // we need the first signed transaction for its signature list so that we know if we need to skip a given signer.
            if signed_transactions
                .first()
                .as_ref()
                .and_then(|it| it.sig_map.as_ref())
                .map_or(false, |it| it.sig_pair.iter().any(|it| pk.starts_with(&it.pub_key_prefix)))
            {
                continue;
            }

            for tx in signed_transactions.to_mut().iter_mut() {
                let sig_map = tx.sig_map.get_or_insert_with(services::SignatureMap::default);
                // todo: reuse `pk_bytes` instead of re-serializing them.
                let sig_pair = super::execute::SignaturePair::from(signer.sign(&tx.body_bytes));

                sig_map.sig_pair.push(sig_pair.into_protobuf());
            }
        }

        match signed_transactions {
            // if it's still borrowed then no signatures have been added (all signers are duplicates).
            Cow::Borrowed(_) => Cow::Borrowed(self),
            Cow::Owned(signed_transactions) => Cow::Owned(Self {
                signed_transactions,
                transactions: OnceCell::new(),
                chunks: self.chunks.clone(),
                transaction_ids: self.transaction_ids.clone(),
                node_ids: self.node_ids.clone(),
                transaction_hashes: self.transaction_hashes.clone(),
            }),
        }
    }

    pub(crate) fn transactions(&self) -> &[services::Transaction] {
        self.transactions.get_or_init(|| {
            self.signed_transactions
                .iter()
                .map(|it| services::Transaction {
                    signed_transaction_bytes: it.encode_to_vec(),
                    ..Default::default()
                })
                .collect()
        })
    }

    pub(crate) fn signed_transactions(&self) -> &[services::SignedTransaction] {
        &self.signed_transactions
    }

    pub(super) fn chunks_len(&self) -> usize {
        self.chunks.len()
    }

    pub(super) fn chunks(&self) -> impl Iterator<Item = SourceChunk<'_>> {
        (0..self.chunks.len()).map(|index| SourceChunk { map: self, index })
    }

    pub(super) fn _transaction_ids(&self) -> &[TransactionId] {
        &self.transaction_ids
    }

    pub(super) fn node_ids(&self) -> &[AccountId] {
        &self.node_ids
    }

    fn transaction_hashes(&self) -> &[TransactionHash] {
        self.transaction_hashes.get_or_init(|| {
            self.signed_transactions.iter().map(|it| TransactionHash::new(&it.body_bytes)).collect()
        })
    }
}
