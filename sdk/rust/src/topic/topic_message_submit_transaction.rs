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

use async_trait::async_trait;
use hedera_proto::services;
use hedera_proto::services::consensus_service_client::ConsensusServiceClient;
use tonic::transport::Channel;

use crate::entity_id::AutoValidateChecksum;
use crate::protobuf::ToProtobuf;
use crate::transaction::{
    AnyTransactionData,
    ToTransactionDataProtobuf,
    TransactionExecute,
};
use crate::{
    AccountId,
    Error,
    LedgerId,
    TopicId,
    Transaction,
    TransactionId,
};

/// Submit a message for consensus.
///
/// Valid and authorized messages on valid topics will be ordered by the consensus service, gossipped to the
/// mirror net, and published (in order) to all subscribers (from the mirror net) on this topic.
///
/// The `submit_key` (if any) must sign this transaction.
///
/// On success, the resulting `TransactionReceipt` contains the topic's updated `topic_sequence_number` and
/// `topic_running_hash`.
///
pub type TopicMessageSubmitTransaction = Transaction<TopicMessageSubmitTransactionData>;

#[cfg_attr(feature = "ffi", serde_with::skip_serializing_none)]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "ffi", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ffi", serde(default, rename_all = "camelCase"))]
pub struct TopicMessageSubmitTransactionData {
    /// The topic ID to submit this message to.
    topic_id: Option<TopicId>,

    /// Message to be submitted.
    /// Max size of the Transaction (including signatures) is 6KiB.
    #[cfg_attr(
        feature = "ffi",
        serde(with = "serde_with::As::<Option<serde_with::base64::Base64>>")
    )]
    message: Option<Vec<u8>>,

    /// The `TransactionId` of the first chunk.
    ///
    /// Should get copied to every subsequent chunk in a fragmented message.
    initial_transaction_id: Option<TransactionId>,

    /// The total number of chunks in the message.
    /// Defaults to 1.
    chunk_total: i32,

    /// The sequence number (from 1 to total) of the current chunk in the message.
    /// Defaults to 1.
    chunk_number: i32,
}

impl Default for TopicMessageSubmitTransactionData {
    fn default() -> Self {
        Self {
            message: None,
            chunk_number: 1,
            chunk_total: 1,
            initial_transaction_id: None,
            topic_id: None,
        }
    }
}

impl TopicMessageSubmitTransaction {
    /// Sets the topic ID to submit this message to.
    pub fn topic_id(&mut self, id: impl Into<TopicId>) -> &mut Self {
        self.body.data.topic_id = Some(id.into());
        self
    }

    /// Sets the message to be submitted.
    pub fn message(&mut self, bytes: impl Into<Vec<u8>>) -> &mut Self {
        self.body.data.message = Some(bytes.into());
        self
    }

    /// Sets the `TransactionId` of the first chunk.
    pub fn initial_transaction_id(&mut self, id: impl Into<TransactionId>) -> &mut Self {
        self.body.data.initial_transaction_id = Some(id.into());
        self
    }

    /// Sets the total number of chunks in the message.
    pub fn chunk_total(&mut self, total: u32) -> &mut Self {
        self.body.data.chunk_total = total as i32;
        self
    }

    /// Sets the sequence number (from 1 to total) of the current chunk in the message.
    pub fn chunk_number(&mut self, number: u32) -> &mut Self {
        self.body.data.chunk_number = number as i32;
        self
    }
}

#[async_trait]
impl TransactionExecute for TopicMessageSubmitTransactionData {
    fn validate_checksums_for_ledger_id(&self, ledger_id: &LedgerId) -> Result<(), Error> {
        self.topic_id.validate_checksum_for_ledger_id(ledger_id)
    }

    async fn execute(
        &self,
        channel: Channel,
        request: services::Transaction,
    ) -> Result<tonic::Response<services::TransactionResponse>, tonic::Status> {
        ConsensusServiceClient::new(channel).submit_message(request).await
    }
}

impl ToTransactionDataProtobuf for TopicMessageSubmitTransactionData {
    fn to_transaction_data_protobuf(
        &self,
        _node_account_id: AccountId,
        _transaction_id: &TransactionId,
    ) -> services::transaction_body::Data {
        let topic_id = self.topic_id.to_protobuf();

        let chunk_info = if let Some(initial_id) = &self.initial_transaction_id {
            let initial_id = initial_id.to_protobuf();

            Some(services::ConsensusMessageChunkInfo {
                initial_transaction_id: Some(initial_id),
                number: self.chunk_number,
                total: self.chunk_total,
            })
        } else {
            None
        };

        services::transaction_body::Data::ConsensusSubmitMessage(
            services::ConsensusSubmitMessageTransactionBody {
                topic_id,
                message: self.message.clone().unwrap_or_default(),
                chunk_info,
            },
        )
    }
}

impl From<TopicMessageSubmitTransactionData> for AnyTransactionData {
    fn from(transaction: TopicMessageSubmitTransactionData) -> Self {
        Self::TopicMessageSubmit(transaction)
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "ffi")]
    mod ffi {
        use std::str::FromStr;

        use assert_matches::assert_matches;

        use crate::transaction::{
            AnyTransaction,
            AnyTransactionData,
        };
        use crate::{
            TopicId,
            TopicMessageSubmitTransaction,
            TransactionId,
        };

        // language=JSON
        const TOPIC_MESSAGE_SUBMIT_EMPTY: &str = r#"{
  "$type": "topicMessageSubmit"
}"#;

        // language=JSON
        const TOPIC_MESSAGE_SUBMIT_TRANSACTION_JSON: &str = r#"{
  "$type": "topicMessageSubmit",
  "topicId": "0.0.1001",
  "message": "TWVzc2FnZQ==",
  "initialTransactionId": "0.0.1001@1656352251.277559886",
  "chunkTotal": 1,
  "chunkNumber": 1
}"#;

        #[test]
        fn it_should_serialize() -> anyhow::Result<()> {
            let mut transaction = TopicMessageSubmitTransaction::new();

            transaction
                .topic_id(TopicId::from(1001))
                .message("Message")
                .initial_transaction_id(TransactionId::from_str("1001@1656352251.277559886")?)
                .chunk_total(1)
                .chunk_number(1);

            let transaction_json = serde_json::to_string_pretty(&transaction)?;

            assert_eq!(transaction_json, TOPIC_MESSAGE_SUBMIT_TRANSACTION_JSON);

            Ok(())
        }

        #[test]
        fn it_should_deserialize() -> anyhow::Result<()> {
            let transaction: AnyTransaction =
                serde_json::from_str(TOPIC_MESSAGE_SUBMIT_TRANSACTION_JSON)?;

            let data = assert_matches!(transaction.body.data, AnyTransactionData::TopicMessageSubmit(transaction) => transaction);

            assert_eq!(data.topic_id.unwrap(), TopicId::from(1001));
            assert_eq!(
                data.initial_transaction_id.unwrap(),
                TransactionId::from_str("1001@1656352251.277559886")?
            );
            assert_eq!(data.chunk_total, 1);
            assert_eq!(data.chunk_number, 1);

            let bytes: Vec<u8> = "Message".into();
            assert_eq!(data.message.unwrap(), bytes);

            Ok(())
        }

        #[test]
        fn it_should_deserialize_empty() -> anyhow::Result<()> {
            let transaction: AnyTransaction = serde_json::from_str(TOPIC_MESSAGE_SUBMIT_EMPTY)?;

            let data = assert_matches!(transaction.body.data, AnyTransactionData::TopicMessageSubmit(transaction) => transaction);

            assert_eq!(data.chunk_number, 1);
            assert_eq!(data.chunk_total, 1);

            Ok(())
        }
    }
}
