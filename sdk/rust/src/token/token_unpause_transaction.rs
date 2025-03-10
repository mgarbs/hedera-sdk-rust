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

use hedera_proto::services;
use hedera_proto::services::token_service_client::TokenServiceClient;
use tonic::transport::Channel;

use crate::protobuf::{
    FromProtobuf,
    ToProtobuf,
};
use crate::transaction::{
    AnyTransactionData,
    ChunkInfo,
    ToSchedulableTransactionDataProtobuf,
    ToTransactionDataProtobuf,
    TransactionData,
    TransactionExecute,
};
use crate::{
    BoxGrpcFuture,
    Error,
    LedgerId,
    TokenId,
    Transaction,
    ValidateChecksums,
};

/// Unpauses the Token. Must be signed with the Token's pause key.
///
/// Once executed the Token is marked as Unpaused and can be used in Transactions.
///
/// The operation is idempotent - becomes a no-op if the Token is already unpaused.
///
/// - If the provided token is not found, the transaction will resolve to `INVALID_TOKEN_ID`.
/// - If the provided token has been deleted, the transaction will resolve to `TOKEN_WAS_DELETED`.
/// - If no Pause Key is defined, the transaction will resolve to `TOKEN_HAS_NO_PAUSE_KEY`.
pub type TokenUnpauseTransaction = Transaction<TokenUnpauseTransactionData>;

#[cfg_attr(feature = "ffi", serde_with::skip_serializing_none)]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "ffi", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ffi", serde(rename_all = "camelCase", default))]
pub struct TokenUnpauseTransactionData {
    /// The token to be unpaused.
    token_id: Option<TokenId>,
}

impl TokenUnpauseTransaction {
    /// Returns the token to be unpaused.
    #[must_use]
    pub fn get_token_id(&self) -> Option<TokenId> {
        self.data().token_id
    }

    /// Sets the token to be unpaused.
    pub fn token_id(&mut self, token_id: impl Into<TokenId>) -> &mut Self {
        self.data_mut().token_id = Some(token_id.into());
        self
    }
}

impl TransactionData for TokenUnpauseTransactionData {}

impl TransactionExecute for TokenUnpauseTransactionData {
    fn execute(
        &self,
        channel: Channel,
        request: services::Transaction,
    ) -> BoxGrpcFuture<'_, services::TransactionResponse> {
        Box::pin(async { TokenServiceClient::new(channel).unpause_token(request).await })
    }
}

impl ValidateChecksums for TokenUnpauseTransactionData {
    fn validate_checksums(&self, ledger_id: &LedgerId) -> Result<(), Error> {
        self.token_id.validate_checksums(ledger_id)
    }
}

impl ToTransactionDataProtobuf for TokenUnpauseTransactionData {
    fn to_transaction_data_protobuf(
        &self,
        chunk_info: &ChunkInfo,
    ) -> services::transaction_body::Data {
        let _ = chunk_info.assert_single_transaction();

        services::transaction_body::Data::TokenUnpause(self.to_protobuf())
    }
}

impl ToSchedulableTransactionDataProtobuf for TokenUnpauseTransactionData {
    fn to_schedulable_transaction_data_protobuf(
        &self,
    ) -> services::schedulable_transaction_body::Data {
        services::schedulable_transaction_body::Data::TokenUnpause(self.to_protobuf())
    }
}

impl From<TokenUnpauseTransactionData> for AnyTransactionData {
    fn from(transaction: TokenUnpauseTransactionData) -> Self {
        Self::TokenUnpause(transaction)
    }
}

impl FromProtobuf<services::TokenUnpauseTransactionBody> for TokenUnpauseTransactionData {
    fn from_protobuf(pb: services::TokenUnpauseTransactionBody) -> crate::Result<Self> {
        Ok(Self { token_id: Option::from_protobuf(pb.token)? })
    }
}

impl ToProtobuf for TokenUnpauseTransactionData {
    type Protobuf = services::TokenUnpauseTransactionBody;

    fn to_protobuf(&self) -> Self::Protobuf {
        services::TokenUnpauseTransactionBody { token: self.token_id.to_protobuf() }
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "ffi")]
    mod ffi {
        use assert_matches::assert_matches;

        use crate::transaction::{
            AnyTransaction,
            AnyTransactionData,
        };
        use crate::{
            TokenId,
            TokenUnpauseTransaction,
        };

        // language=JSON
        const TOKEN_UNPAUSE_TRANSACTION_JSON: &str = r#"{
  "$type": "tokenUnpause",
  "tokenId": "0.0.1001"
}"#;

        #[test]
        fn it_should_serialize() -> anyhow::Result<()> {
            let mut transaction = TokenUnpauseTransaction::new();

            transaction.token_id(TokenId::from(1001));

            let transaction_json = serde_json::to_string_pretty(&transaction)?;

            assert_eq!(transaction_json, TOKEN_UNPAUSE_TRANSACTION_JSON);

            Ok(())
        }

        #[test]
        fn it_should_deserialize() -> anyhow::Result<()> {
            let transaction: AnyTransaction = serde_json::from_str(TOKEN_UNPAUSE_TRANSACTION_JSON)?;

            let data = assert_matches!(transaction.data(), AnyTransactionData::TokenUnpause(transaction) => transaction);

            assert_eq!(data.token_id.unwrap(), TokenId::from(1001));

            Ok(())
        }
    }
}
