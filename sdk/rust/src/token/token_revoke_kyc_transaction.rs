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
    AccountId,
    BoxGrpcFuture,
    Error,
    LedgerId,
    TokenId,
    Transaction,
    ValidateChecksums,
};

/// Revokes KYC from the account for the given token.
///
/// Must be signed by the Token's kycKey.
///
/// Once executed the Account is marked as KYC Revoked.
///
/// - If the provided account is not found, the transaction will resolve to `INVALID_ACCOUNT_ID`.
/// - If the provided account has been deleted, the transaction will resolve to `ACCOUNT_DELETED`.
/// - If the provided token is not found, the transaction will resolve to `INVALID_TOKEN_ID`.
/// - If the provided token has been deleted, the transaction will resolve to `TOKEN_WAS_DELETED`.
/// - If an Association between the provided token and account is not found, the transaction will
/// resolve to `TOKEN_NOT_ASSOCIATED_TO_ACCOUNT`.
/// - If no KYC Key is defined, the transaction will resolve to `TOKEN_HAS_NO_KYC_KEY`.
pub type TokenRevokeKycTransaction = Transaction<TokenRevokeKycTransactionData>;

#[cfg_attr(feature = "ffi", serde_with::skip_serializing_none)]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "ffi", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ffi", serde(rename_all = "camelCase", default))]
pub struct TokenRevokeKycTransactionData {
    /// The account to have their KYC revoked.
    account_id: Option<AccountId>,

    /// The token for which this account will have their KYC revoked.
    token_id: Option<TokenId>,
}

impl TokenRevokeKycTransaction {
    /// Returns the account to have their KYC revoked.
    #[must_use]
    pub fn get_account_id(&self) -> Option<AccountId> {
        self.data().account_id
    }
    /// Sets the account to have their KYC revoked.
    pub fn account_id(&mut self, account_id: AccountId) -> &mut Self {
        self.data_mut().account_id = Some(account_id);
        self
    }

    /// Returns the token for which the account will have their KYC revoked.
    #[must_use]
    pub fn get_token_id(&self) -> Option<TokenId> {
        self.data().token_id
    }

    /// Sets the token for which this account will have their KYC revoked.
    pub fn token_id(&mut self, token_id: impl Into<TokenId>) -> &mut Self {
        self.data_mut().token_id = Some(token_id.into());
        self
    }
}

impl TransactionData for TokenRevokeKycTransactionData {}

impl TransactionExecute for TokenRevokeKycTransactionData {
    fn execute(
        &self,
        channel: Channel,
        request: services::Transaction,
    ) -> BoxGrpcFuture<'_, services::TransactionResponse> {
        Box::pin(async {
            TokenServiceClient::new(channel).revoke_kyc_from_token_account(request).await
        })
    }
}

impl ValidateChecksums for TokenRevokeKycTransactionData {
    fn validate_checksums(&self, ledger_id: &LedgerId) -> Result<(), Error> {
        self.token_id.validate_checksums(ledger_id)?;
        self.account_id.validate_checksums(ledger_id)
    }
}

impl ToTransactionDataProtobuf for TokenRevokeKycTransactionData {
    fn to_transaction_data_protobuf(
        &self,
        chunk_info: &ChunkInfo,
    ) -> services::transaction_body::Data {
        let _ = chunk_info.assert_single_transaction();

        services::transaction_body::Data::TokenRevokeKyc(self.to_protobuf())
    }
}

impl ToSchedulableTransactionDataProtobuf for TokenRevokeKycTransactionData {
    fn to_schedulable_transaction_data_protobuf(
        &self,
    ) -> services::schedulable_transaction_body::Data {
        services::schedulable_transaction_body::Data::TokenRevokeKyc(self.to_protobuf())
    }
}

impl From<TokenRevokeKycTransactionData> for AnyTransactionData {
    fn from(transaction: TokenRevokeKycTransactionData) -> Self {
        Self::TokenRevokeKyc(transaction)
    }
}

impl FromProtobuf<services::TokenRevokeKycTransactionBody> for TokenRevokeKycTransactionData {
    fn from_protobuf(pb: services::TokenRevokeKycTransactionBody) -> crate::Result<Self> {
        Ok(Self {
            account_id: Option::from_protobuf(pb.account)?,
            token_id: Option::from_protobuf(pb.token)?,
        })
    }
}

impl ToProtobuf for TokenRevokeKycTransactionData {
    type Protobuf = services::TokenRevokeKycTransactionBody;

    fn to_protobuf(&self) -> Self::Protobuf {
        services::TokenRevokeKycTransactionBody {
            token: self.token_id.to_protobuf(),
            account: self.account_id.to_protobuf(),
        }
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
            AccountId,
            TokenId,
            TokenRevokeKycTransaction,
        };

        // language=JSON
        const TOKEN_REVOKE_KYC_TRANSACTION_JSON: &str = r#"{
  "$type": "tokenRevokeKyc",
  "accountId": "0.0.1001",
  "tokenId": "0.0.1002"
}"#;

        #[test]
        fn it_should_serialize() -> anyhow::Result<()> {
            let mut transaction = TokenRevokeKycTransaction::new();

            transaction.account_id(AccountId::from(1001)).token_id(TokenId::from(1002));

            let transaction_json = serde_json::to_string_pretty(&transaction)?;

            assert_eq!(transaction_json, TOKEN_REVOKE_KYC_TRANSACTION_JSON);

            Ok(())
        }

        #[test]
        fn it_should_deserialize() -> anyhow::Result<()> {
            let transaction: AnyTransaction =
                serde_json::from_str(TOKEN_REVOKE_KYC_TRANSACTION_JSON)?;

            let data = assert_matches!(transaction.data(), AnyTransactionData::TokenRevokeKyc(transaction) => transaction);

            assert_eq!(data.token_id, Some(TokenId::from(1002)));
            assert_eq!(data.account_id, Some(AccountId::from(1001)));

            Ok(())
        }
    }
}
