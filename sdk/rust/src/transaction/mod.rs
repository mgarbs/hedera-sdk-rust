/*
 * ‌
 * Hedera Rust SDK
 * ​
 * Copyright (C) 2022 - 2023 Hedera Hashgraph, LLC
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

// serde appears to insert the bound twice.
#![cfg_attr(feature = "ffi", allow(clippy::type_repetition_in_bounds))]

use std::borrow::Cow;
use std::fmt;
use std::fmt::{
    Debug,
    Formatter,
};
use std::num::NonZeroUsize;

use hedera_proto::services;
use prost::Message;
use time::Duration;

use crate::execute::execute;
use crate::signer::AnySigner;
use crate::{
    AccountId,
    Client,
    Error,
    Hbar,
    Operator,
    PrivateKey,
    PublicKey,
    Signer,
    TransactionId,
    TransactionResponse,
    ValidateChecksums,
};

mod any;
mod chunked;
mod execute;
mod protobuf;
mod source;
#[cfg(test)]
mod tests;

pub use any::AnyTransaction;
#[cfg(feature = "ffi")]
pub(crate) use any::AnyTransactionBody;
pub(crate) use any::AnyTransactionData;
pub(crate) use chunked::{
    ChunkData,
    ChunkInfo,
    ChunkedTransactionData,
};
#[cfg(feature = "ffi")]
pub(crate) use execute::SourceTransaction;
pub(crate) use execute::{
    TransactionData,
    TransactionExecute,
    TransactionExecuteChunked,
};
pub(crate) use protobuf::{
    ToSchedulableTransactionDataProtobuf,
    ToTransactionDataProtobuf,
};
pub(crate) use source::TransactionSources;

const DEFAULT_TRANSACTION_VALID_DURATION: Duration = Duration::seconds(120);

/// A transaction that can be executed on the Hedera network.
#[cfg_attr(feature = "ffi", derive(serde::Serialize))]
#[cfg_attr(feature = "ffi", serde(rename_all = "camelCase"))]
pub struct Transaction<D> {
    #[cfg_attr(feature = "ffi", serde(flatten))]
    #[cfg_attr(feature = "ffi", serde(bound = "D: Into<AnyTransactionData> + Clone"))]
    body: TransactionBody<D>,

    #[cfg_attr(feature = "ffi", serde(skip))]
    signers: Vec<AnySigner>,

    #[cfg_attr(feature = "ffi", serde(skip))]
    sources: Option<TransactionSources>,
}

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "ffi", derive(serde::Serialize))]
#[cfg_attr(feature = "ffi", serde(rename_all = "camelCase"))]
pub(crate) struct TransactionBody<D> {
    #[cfg_attr(feature = "ffi", serde(flatten))]
    #[cfg_attr(
        feature = "ffi",
        serde(
            with = "serde_with::As::<serde_with::FromInto<AnyTransactionData>>",
            bound = "D: Into<AnyTransactionData> + Clone"
        )
    )]
    pub(crate) data: D,

    #[cfg_attr(feature = "ffi", serde(skip_serializing_if = "Option::is_none"))]
    pub(crate) node_account_ids: Option<Vec<AccountId>>,

    #[cfg_attr(
        feature = "ffi",
        serde(
            with = "serde_with::As::<Option<serde_with::DurationSeconds<i64>>>",
            skip_serializing_if = "Option::is_none"
        )
    )]
    pub(crate) transaction_valid_duration: Option<Duration>,

    #[cfg_attr(feature = "ffi", serde(skip_serializing_if = "Option::is_none"))]
    pub(crate) max_transaction_fee: Option<Hbar>,

    #[cfg_attr(feature = "ffi", serde(skip_serializing_if = "String::is_empty"))]
    pub(crate) transaction_memo: String,

    #[cfg_attr(feature = "ffi", serde(skip_serializing_if = "Option::is_none"))]
    pub(crate) transaction_id: Option<TransactionId>,

    #[cfg_attr(feature = "ffi", serde(skip_serializing_if = "Option::is_none"))]
    pub(crate) operator: Option<Operator>,

    #[cfg_attr(feature = "ffi", serde(skip_serializing_if = "std::ops::Not::not"))]
    pub(crate) is_frozen: bool,
}

impl<D> Default for Transaction<D>
where
    D: Default,
{
    fn default() -> Self {
        Self {
            body: TransactionBody {
                data: D::default(),
                node_account_ids: None,
                transaction_valid_duration: None,
                max_transaction_fee: None,
                transaction_memo: String::new(),
                transaction_id: None,
                operator: None,
                is_frozen: false,
            },
            signers: Vec::new(),
            sources: None,
        }
    }
}

impl<D> Debug for Transaction<D>
where
    D: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Transaction").field("body", &self.body).finish()
    }
}

impl<D> Transaction<D>
where
    D: Default,
{
    /// Create a new default transaction.
    ///
    /// Does the same thing as [`default`](Self::default)
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl<D> Transaction<D> {
    pub(crate) fn from_parts(body: TransactionBody<D>, signers: Vec<AnySigner>) -> Self {
        Self { body, signers, sources: None }
    }

    pub(crate) fn is_frozen(&self) -> bool {
        self.body.is_frozen
    }

    pub(crate) fn signers(&self) -> impl Iterator<Item = &AnySigner> {
        self.signers.iter()
    }

    pub(crate) fn sources(&self) -> Option<&TransactionSources> {
        self.sources.as_ref()
    }

    /// # Panics
    /// If `self.is_frozen()`.
    #[track_caller]
    pub(crate) fn require_not_frozen(&self) {
        assert!(
            !self.is_frozen(),
            "transaction is immutable; it has at least one signature or has been explicitly frozen"
        );
    }

    #[cfg(feature = "ffi")]
    pub(crate) fn body(&self) -> &TransactionBody<D> {
        &self.body
    }

    /// # Panics
    /// If `self.is_frozen()`.
    fn body_mut(&mut self) -> &mut TransactionBody<D> {
        self.require_not_frozen();
        &mut self.body
    }

    pub(crate) fn into_body(self) -> TransactionBody<D> {
        self.body
    }

    pub(crate) fn data(&self) -> &D {
        &self.body.data
    }

    /// # Panics
    /// If `self.is_frozen()`.
    pub(crate) fn data_mut(&mut self) -> &mut D {
        self.require_not_frozen();
        &mut self.body.data
    }

    /// Returns the account IDs of the nodes that this transaction may be submitted to.
    ///
    /// `None` means any node configured on the client.
    #[must_use]
    pub fn get_node_account_ids(&self) -> Option<&[AccountId]> {
        self.body.node_account_ids.as_deref()
    }

    /// Sets the account IDs of the nodes that this transaction may be submitted to.
    ///
    /// Defaults to the full list of nodes configured on the client.
    #[track_caller]
    pub fn node_account_ids(&mut self, ids: impl IntoIterator<Item = AccountId>) -> &mut Self {
        self.body_mut().node_account_ids = Some(ids.into_iter().collect());
        self
    }

    /// Returns the duration that this transaction is valid for, once finalized and signed.
    #[must_use]
    pub fn get_transaction_valid_duration(&self) -> Option<Duration> {
        self.body.transaction_valid_duration
    }

    /// Sets the duration that this transaction is valid for, once finalized and signed.
    ///
    /// Defaults to 120 seconds (or two minutes).
    pub fn transaction_valid_duration(&mut self, duration: Duration) -> &mut Self {
        self.body_mut().transaction_valid_duration = Some(duration);
        self
    }

    /// Returns the maximum transaction fee the paying account is willing to pay.
    #[must_use]
    pub fn get_max_transaction_fee(&self) -> Option<Hbar> {
        self.body.max_transaction_fee
    }

    /// Sets the maximum transaction fee the paying account is willing to pay.
    pub fn max_transaction_fee(&mut self, fee: Hbar) -> &mut Self {
        self.body_mut().max_transaction_fee = Some(fee);
        self
    }

    /// Sets a note / description that should be recorded in the transaction record.
    ///
    /// Maximum length of 100 characters.
    #[must_use]
    pub fn get_transaction_memo(&self) -> &str {
        &self.body.transaction_memo
    }

    /// Sets a note or description that should be recorded in the transaction record.
    ///
    /// Maximum length of 100 characters.
    pub fn transaction_memo(&mut self, memo: impl AsRef<str>) -> &mut Self {
        self.body_mut().transaction_memo = memo.as_ref().to_owned();
        self
    }

    /// Returns the explicit transaction ID to use to identify this transaction.
    ///
    /// Overrides the payer account defined on this transaction or on the client.
    #[must_use]
    pub fn get_transaction_id(&self) -> Option<TransactionId> {
        self.body.transaction_id
    }

    /// Sets an explicit transaction ID to use to identify this transaction.
    ///
    /// Overrides the payer account defined on this transaction or on the client.
    pub fn transaction_id(&mut self, id: TransactionId) -> &mut Self {
        self.body_mut().transaction_id = Some(id);
        self
    }

    /// Sign the transaction.
    pub fn sign(&mut self, private_key: PrivateKey) -> &mut Self {
        self.sign_signer(AnySigner::PrivateKey(private_key))
    }

    /// Sign the transaction.
    pub fn sign_with(&mut self, public_key: PublicKey, signer: Signer) -> &mut Self {
        self.sign_signer(AnySigner::Arbitrary(Box::new(public_key), signer))
    }

    pub(crate) fn sign_signer(&mut self, signer: AnySigner) -> &mut Self {
        // We're _supposed_ to require frozen here, but really there's no reason I can think of to do that.

        // skip the signer if we already have it.
        if self.signers.iter().any(|it| it.public_key() == signer.public_key()) {
            return self;
        }

        self.signers.push(signer);
        self
    }
}

impl<D: ChunkedTransactionData> Transaction<D> {
    /// Returns the maximum number of chunks this transaction will be split into.
    #[must_use]
    pub fn get_max_chunks(&self) -> usize {
        self.data().chunk_data().max_chunks
    }

    /// Sets the maximum number of chunks this transaction will be split into.
    pub fn max_chunks(&mut self, max_chunks: usize) -> &mut Self {
        self.data_mut().chunk_data_mut().max_chunks = max_chunks;

        self
    }

    // todo: just return a `NonZeroUsize` instead? Take something along the lines of a `u32`?
    /// Returns the maximum size of any chunk.
    pub fn get_chunk_size(&self) -> usize {
        self.data().chunk_data().chunk_size.get()
    }

    // todo: just take a `NonZeroUsize` instead? Take something along the lines of a `u32`?
    /// Sets the maximum size of any chunk.
    ///
    /// # Panics
    /// If `size` == 0
    pub fn chunk_size(&mut self, size: usize) -> &mut Self {
        let Some(size) = NonZeroUsize::new(size) else {
            panic!("Cannot set chunk-size to zero")
        };

        self.data_mut().chunk_data_mut().chunk_size = size;

        self
    }
}

impl<D: ValidateChecksums> Transaction<D> {
    /// Freeze the transaction so that no further modifications can be made.
    ///
    /// # Errors
    /// - [`Error::FreezeUnsetNodeAccountIds`] if no [`node_account_ids`](Self::node_account_ids) were set.
    pub fn freeze(&mut self) -> crate::Result<&mut Self> {
        self.freeze_with(None)
    }

    /// Freeze the transaction so that no further modifications can be made.
    ///
    /// # Errors
    /// - [`Error::FreezeUnsetNodeAccountIds`] if no [`node_account_ids`](Self::node_account_ids) were set and `client.is_none()`.
    /// - [`Error::CannotPerformTaskWithoutLedgerId`] if [`auto_validate_checksums`](Client::auto_validate_checksums)
    ///    is enabled on the client and the client has no ledger id.
    pub fn freeze_with<'a>(
        &mut self,
        client: impl Into<Option<&'a Client>>,
    ) -> crate::Result<&mut Self> {
        if self.is_frozen() {
            return Ok(self);
        }
        let client: Option<&Client> = client.into();

        let node_account_ids = match &self.body.node_account_ids {
            // the clone here is the lesser of two evils.
            Some(it) => it.clone(),
            None => client.ok_or(Error::FreezeUnsetNodeAccountIds)?.random_node_ids(),
        };

        // note to reviewer: this is intentionally still an option, fallback is used later, swift doesn't *have* default max transaction fee and fixing it is a massive PITA.
        let max_transaction_fee = self.body.max_transaction_fee.or_else(|| {
            // no max has been set on the *transaction*
            // check if there is a global max set on the client
            let client_max_transaction_fee = client
                .map(|it| it.max_transaction_fee().load(std::sync::atomic::Ordering::Relaxed));

            match client_max_transaction_fee {
                Some(max) if max > 1 => Some(Hbar::from_tinybars(max as i64)),
                // no max has been set on the client either
                // fallback to the hard-coded default for this transaction type
                _ => None,
            }
        });

        let operator = client.and_then(|it| it.operator_internal().as_deref().cloned());

        // note: yes, there's an `Some(opt.unwrap())`, this is INTENTIONAL.
        self.body.node_account_ids = Some(node_account_ids);
        self.body.max_transaction_fee = max_transaction_fee;
        self.body.operator = operator;
        self.body.is_frozen = true;

        if let Some(client) = client {
            if client.auto_validate_checksums() {
                if let Some(ledger_id) = &*client.ledger_id_internal() {
                    self.validate_checksums(ledger_id)?;
                } else {
                    return Err(crate::Error::CannotPerformTaskWithoutLedgerId {
                        task: "validate checksums",
                    });
                }
            }
        }

        Ok(self)
    }
}

impl<D: TransactionExecute> Transaction<D> {
    fn make_transaction_list(&self) -> crate::Result<Vec<services::Transaction>> {
        assert!(self.is_frozen());

        // todo: fix this with chunked transactions.
        let initial_transaction_id = match self.get_transaction_id() {
            Some(id) => id,
            None => self
                .body
                .operator
                .as_ref()
                .ok_or(crate::Error::NoPayerAccountOrTransactionId)?
                .generate_transaction_id(),
        };

        let transaction_list = {
            let used_chunks = self.data().maybe_chunk_data().map_or(1, ChunkData::used_chunks);
            let node_account_ids = self.body.node_account_ids.as_deref().unwrap();

            let mut transaction_list = Vec::with_capacity(used_chunks * node_account_ids.len());

            // Note: This ordering is *important*,
            // there's no documentation for it but `TransactionList` is sorted by chunk number,
            // then `node_id` (in the order they were added to the transaction)
            for chunk in 0..used_chunks {
                let current_transaction_id = match chunk {
                    0 => initial_transaction_id,
                    _ => self
                        .body
                        .operator
                        .as_ref()
                        .ok_or(crate::Error::NoPayerAccountOrTransactionId)?
                        .generate_transaction_id(),
                };

                for node_account_id in node_account_ids.iter().copied() {
                    let chunk_info = ChunkInfo {
                        current: chunk,
                        total: used_chunks,
                        initial_transaction_id,
                        current_transaction_id,
                        node_account_id,
                    };

                    transaction_list.push(self.make_request_inner(&chunk_info).0);
                }
            }

            transaction_list
        };

        Ok(transaction_list)
    }

    pub(crate) fn make_sources(&self) -> crate::Result<Cow<'_, TransactionSources>> {
        assert!(self.is_frozen());

        if let Some(sources) = &self.sources {
            return Ok(sources.sign_with(&self.signers));
        }

        return Ok(Cow::Owned(TransactionSources::new(self.make_transaction_list()?).unwrap()));
    }

    /// Convert `self` to protobuf encoded bytes.
    ///
    /// # Errors
    /// - If `freeze_with` wasn't called with an operator.
    ///
    /// # Panics
    /// - If `!self.is_frozen()`.
    pub fn to_bytes(&self) -> crate::Result<Vec<u8>> {
        assert!(self.is_frozen(), "Transaction must be frozen to call `to_bytes`");

        let transaction_list = self
            .sources
            .as_ref()
            .map_or_else(|| self.make_transaction_list(), |it| Ok(it.transactions().to_vec()))?;

        Ok(hedera_proto::sdk::TransactionList { transaction_list }.encode_to_vec())
    }

    pub(crate) fn add_signature_signer(&mut self, signer: &AnySigner) {
        assert!(self.is_frozen());

        // note: the following pair of cheecks are for more detailed panic messages
        // IE, they should *hopefully* be tripped first
        assert_eq!(
            self.body.node_account_ids.as_deref().map_or(0, <[AccountId]>::len),
            1,
            "cannot manually add a signature to a transaction with multiple nodes"
        );

        if let Some(chunk_data) = self.data().maybe_chunk_data() {
            assert!(
                chunk_data.used_chunks() <= 1,
                "cannot manually add a signature to a chunked transaction with multiple chunks (message length `{}` > chunk size `{}`)",
                chunk_data.data.len(),
                chunk_data.chunk_size
            );
        }

        let sources = self.make_sources().unwrap();

        // this is the only check that is for correctness rather than debugability.
        assert!(sources.transactions().len() == 1);

        let sources = sources.sign_with(std::slice::from_ref(signer));

        // if we have a `Cow::Borrowed` that'd mean there was no modification
        if let Cow::Owned(sources) = sources {
            self.sources = Some(sources);
        }
    }

    // todo: make this public... :/
    // todo: should this return `Result<&mut Self>`?
    pub(crate) fn _add_signature(&mut self, pk: PublicKey, signature: Vec<u8>) -> &mut Self {
        self.add_signature_signer(&AnySigner::Arbitrary(
            Box::new(pk),
            Box::new(move |_| signature.clone()),
        ));

        self
    }
}

impl<D> Transaction<D>
where
    D: TransactionExecute,
{
    /// Execute this transaction against the provided client of the Hedera network.
    // todo:
    pub async fn execute(&mut self, client: &Client) -> crate::Result<TransactionResponse> {
        self.execute_with_optional_timeout(client, None).await
    }

    pub(crate) async fn execute_with_optional_timeout(
        &mut self,
        client: &Client,
        timeout: Option<std::time::Duration>,
    ) -> crate::Result<TransactionResponse> {
        // it's fine to call freeze while already frozen, so, let `freeze_with` handle the freeze check.
        self.freeze_with(Some(client))?;

        if let Some(sources) = &self.sources {
            return self::execute::SourceTransaction::new(self, sources)
                .execute(client, timeout)
                .await;
        }

        if let Some(chunk_data) = self.data().maybe_chunk_data() {
            // todo: log a warning: user actually wanted `execute_all`.
            // instead of `panic`king we just pretend we were `execute_all` and
            // return the first result (*after* executing all the transactions).
            return self
                .execute_all_inner(chunk_data, client, timeout)
                .await
                .map(|mut it| it.swap_remove(0));
        }

        execute(client, self, timeout).await
    }

    // this is in *this* impl block rather than the `: TransactionExecuteChunked` impl block
    //because there's the off chance that someone calls `execute` on a Transaction that wants `execute_all`...
    async fn execute_all_inner(
        &self,
        chunk_data: &ChunkData,
        client: &Client,
        timeout_per_chunk: Option<std::time::Duration>,
    ) -> crate::Result<Vec<TransactionResponse>> {
        assert!(self.is_frozen());
        let wait_for_receipts = self.data().wait_for_receipt();

        // fixme: error with an actual error.
        #[allow(clippy::manual_assert)]
        if chunk_data.data.len() > chunk_data.max_message_len() {
            todo!("error: message too big")
        }

        let used_chunks = chunk_data.used_chunks();

        let mut responses = Vec::with_capacity(chunk_data.used_chunks());

        let initial_transaction_id = {
            let resp = execute(
                client,
                &chunked::FirstChunkView { transaction: self, total_chunks: used_chunks },
                timeout_per_chunk,
            )
            .await?;

            if wait_for_receipts {
                // todo `get_receipt_with_optional_timeout`.
                resp.get_receipt(client).await?;
            }

            let initial_transaction_id = resp.transaction_id;
            responses.push(resp);

            initial_transaction_id
        };

        for chunk in 1..used_chunks {
            let resp = execute(
                client,
                &chunked::ChunkView {
                    transaction: self,
                    initial_transaction_id,
                    current_chunk: chunk,
                    total_chunks: used_chunks,
                },
                timeout_per_chunk,
            )
            .await?;

            if wait_for_receipts {
                // todo `get_receipt_with_optional_timeout`.
                resp.get_receipt(client).await?;
            }

            responses.push(resp);
        }

        Ok(responses)
    }

    /// Execute this transaction against the provided client of the Hedera network.
    // todo:
    #[allow(clippy::missing_errors_doc)]
    pub async fn execute_with_timeout(
        &mut self,
        client: &Client,
        // fixme: be consistent with `time::Duration`? Except `tokio::time` is `std::time`, and we depend on tokio.
        timeout: std::time::Duration,
    ) -> crate::Result<TransactionResponse> {
        self.execute_with_optional_timeout(client, Some(timeout)).await
    }
}

impl<D> Transaction<D>
where
    D: TransactionExecuteChunked,
{
    /// Execute all transactions against the provided client of the Hedera network.
    pub async fn execute_all(
        &mut self,
        client: &Client,
    ) -> crate::Result<Vec<TransactionResponse>> {
        self.execute_all_with_optional_timeout(client, None).await
    }

    pub(crate) async fn execute_all_with_optional_timeout(
        &mut self,
        client: &Client,
        timeout_per_chunk: Option<std::time::Duration>,
    ) -> crate::Result<Vec<TransactionResponse>> {
        // it's fine to call freeze while already frozen, so, let `freeze_with` handle the freeze check.
        self.freeze_with(Some(client))?;

        // fixme: dedup this with `execute_with_optional_timeout`
        if let Some(sources) = &self.sources {
            return self::execute::SourceTransaction::new(self, sources)
                .execute_all(client, timeout_per_chunk)
                .await;
        }

        // sorry for the mess: this can technically infinite loop
        // (it won't, the loop condition would be dependent on chunk_data somehow being `Some` and `None` at the same time).
        let Some(chunk_data) = self.data().maybe_chunk_data() else {
            return Ok(Vec::from([self.execute_with_optional_timeout(client, timeout_per_chunk).await?]))
        };

        self.execute_all_inner(chunk_data, client, timeout_per_chunk).await
    }
}

// these impls are on `AnyTransaction`, but they're here instead of in `any` because actually implementing them is only possible here.
impl AnyTransaction {
    /// # Examples
    /// ```
    /// # fn main() -> hedera::Result<()> {
    /// use hedera::AnyTransaction;
    /// let bytes = hex::decode(concat!(
    ///     "0a522a500a4c0a120a0c0885c8879e0610a8bdd9840312021865120218061880",
    ///     "94ebdc0322020877320c686920686173686772617068721a0a180a0a0a021802",
    ///     "108088debe010a0a0a02186510ff87debe0112000a522a500a4c0a120a0c0885",
    ///     "c8879e0610a8bdd984031202186512021807188094ebdc0322020877320c6869",
    ///     "20686173686772617068721a0a180a0a0a021802108088debe010a0a0a021865",
    ///     "10ff87debe011200"
    /// )).unwrap();
    /// let tx = AnyTransaction::from_bytes(&bytes)?;
    /// # let _ = tx;
    /// # Ok(())
    /// # }
    /// ```
    /// # Errors
    /// - [`Error::FromProtobuf`] if a valid transaction cannot be parsed from the bytes.
    #[allow(deprecated)]
    pub fn from_bytes(bytes: &[u8]) -> crate::Result<Self> {
        let list =
            hedera_proto::sdk::TransactionList::decode(bytes).map_err(Error::from_protobuf)?;

        let list = if list.transaction_list.is_empty() {
            Vec::from([services::Transaction::decode(bytes).map_err(Error::from_protobuf)?])
        } else {
            list.transaction_list
        };

        let sources = TransactionSources::new(list)?;

        let transaction_bodies: Result<Vec<_>, _> = sources
            .signed_transactions()
            .iter()
            .map(|it| {
                services::TransactionBody::decode(&*it.body_bytes).map_err(Error::from_protobuf)
            })
            .collect();

        let transaction_bodies = transaction_bodies?;
        {
            let (first, transaction_bodies) = transaction_bodies
                .split_first()
                .ok_or_else(|| Error::from_protobuf("no transactions found"))?;

            for it in transaction_bodies.iter() {
                if !pb_transaction_body_eq(first, it) {
                    return Err(Error::from_protobuf("transaction parts unexpectedly unequal"));
                }
            }
        }

        // todo: reuse work
        let transaction_data = {
            let data: Result<_, _> = sources
                .chunks()
                .filter_map(|it| it.signed_transactions().first())
                .map(|it| {
                    services::TransactionBody::decode(&*it.body_bytes)
                        .map_err(Error::from_protobuf)
                        .and_then(|pb| pb_getf!(pb, data))
                })
                .collect();

            data?
        };

        // note: this creates the transaction in a frozen state.
        let mut res = Self::from_protobuf(transaction_bodies[0].clone(), transaction_data)?;

        // note: this doesn't check freeze for obvious reasons.
        res.body.node_account_ids = Some(sources.node_ids().to_vec());
        res.sources = Some(sources);

        Ok(res)
    }
}

/// Returns `true` if lhs == rhs other than `transaction_id` and `node_account_id`, `false` otherwise.
#[allow(deprecated)]
fn pb_transaction_body_eq(
    lhs: &services::TransactionBody,
    rhs: &services::TransactionBody,
) -> bool {
    // destructure one side to ensure we don't miss any fields.
    let services::TransactionBody {
        transaction_id: _,
        node_account_id: _,
        transaction_fee,
        transaction_valid_duration,
        generate_record,
        memo,
        data,
    } = rhs;

    if &lhs.transaction_fee != transaction_fee {
        return false;
    }

    if &lhs.transaction_valid_duration != transaction_valid_duration {
        return false;
    }

    if &lhs.generate_record != generate_record {
        return false;
    }

    if &lhs.memo != memo {
        return false;
    }

    match (&lhs.data, data) {
        (None, None) => {}
        (Some(lhs), Some(rhs)) => match (lhs, rhs) {
            (
                services::transaction_body::Data::ConsensusSubmitMessage(lhs),
                services::transaction_body::Data::ConsensusSubmitMessage(rhs),
            ) => {
                let services::ConsensusSubmitMessageTransactionBody {
                    topic_id,
                    message: _,
                    chunk_info,
                } = rhs;

                if &lhs.topic_id != topic_id {
                    return false;
                }

                match (lhs.chunk_info.as_ref(), chunk_info.as_ref()) {
                    (None, None) => {}
                    (Some(lhs), Some(rhs)) => {
                        let services::ConsensusMessageChunkInfo {
                            initial_transaction_id,
                            total,
                            number: _,
                        } = rhs;

                        if &lhs.initial_transaction_id != initial_transaction_id {
                            return false;
                        }

                        if &lhs.total != total {
                            return false;
                        }
                    }
                    (Some(_), None) | (None, Some(_)) => return false,
                }
            }
            (
                services::transaction_body::Data::FileAppend(lhs),
                services::transaction_body::Data::FileAppend(rhs),
            ) => {
                let services::FileAppendTransactionBody { file_id, contents: _ } = rhs;

                if &lhs.file_id != file_id {
                    return false;
                }
            }
            (_, _) if lhs != rhs => return false,
            _ => {}
        },
        (Some(_), None) | (None, Some(_)) => return false,
    }

    true
}
