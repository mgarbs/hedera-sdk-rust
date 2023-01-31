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
use hedera_proto::services::smart_contract_service_client::SmartContractServiceClient;
use tonic::transport::Channel;

use crate::protobuf::{
    FromProtobuf,
    ToProtobuf,
};
use crate::transaction::{
    AnyTransactionData,
    ToTransactionDataProtobuf,
    TransactionExecute,
};
use crate::{
    AccountId,
    BoxGrpcFuture,
    ContractId,
    Error,
    LedgerId,
    Transaction,
    ValidateChecksums,
};

/// Marks a contract as deleted and transfers its remaining hBars, if any, to
/// a designated receiver.
///
pub type ContractDeleteTransaction = Transaction<ContractDeleteTransactionData>;

#[cfg_attr(feature = "ffi", serde_with::skip_serializing_none)]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "ffi", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ffi", serde(rename_all = "camelCase", default))]
pub struct ContractDeleteTransactionData {
    contract_id: Option<ContractId>,

    transfer_account_id: Option<AccountId>,

    transfer_contract_id: Option<ContractId>,
}

impl ContractDeleteTransaction {
    /// Returns the ID of the contract that should be deleted.
    #[must_use]
    pub fn get_contract_id(&self) -> Option<ContractId> {
        self.data().contract_id
    }

    /// Sets ID of the contract which should be deleted.
    pub fn contract_id(&mut self, id: ContractId) -> &mut Self {
        self.data_mut().contract_id = Some(id);
        self
    }

    /// Returns the ID of the account which will receive all remaining hbars.
    #[must_use]
    pub fn get_transfer_account_id(&self) -> Option<AccountId> {
        self.data().transfer_account_id
    }

    /// Sets the ID of the account which will receive all remaining hbars.
    pub fn transfer_account_id(&mut self, id: AccountId) -> &mut Self {
        self.data_mut().transfer_account_id = Some(id);
        self
    }

    /// Returns ID of the contract which will receive all rmaining hbars.
    #[must_use]
    pub fn get_transfer_contract_id(&self) -> Option<ContractId> {
        self.data().transfer_contract_id
    }

    /// Sets the the ID of the contract which will receive all remaining hbars.
    pub fn transfer_contract_id(&mut self, id: ContractId) -> &mut Self {
        self.data_mut().transfer_contract_id = Some(id);
        self
    }
}

impl TransactionExecute for ContractDeleteTransactionData {
    fn execute(
        &self,
        channel: Channel,
        request: services::Transaction,
    ) -> BoxGrpcFuture<'_, services::TransactionResponse> {
        Box::pin(async { SmartContractServiceClient::new(channel).delete_contract(request).await })
    }
}

impl ValidateChecksums for ContractDeleteTransactionData {
    fn validate_checksums(&self, ledger_id: &LedgerId) -> Result<(), Error> {
        self.contract_id.validate_checksums(ledger_id)?;
        self.transfer_account_id.validate_checksums(ledger_id)?;
        self.transfer_contract_id.validate_checksums(ledger_id)
    }
}

impl ToTransactionDataProtobuf for ContractDeleteTransactionData {
    fn to_transaction_data_protobuf(
        &self,
        _node_account_id: AccountId,
        _transaction_id: &crate::TransactionId,
    ) -> services::transaction_body::Data {
        let delete_contract_id = self.contract_id.to_protobuf();

        let obtainers = match (&self.transfer_account_id, &self.transfer_contract_id) {
            (Some(account_id), None) => {
                Some(services::contract_delete_transaction_body::Obtainers::TransferAccountId(
                    account_id.to_protobuf(),
                ))
            }

            (None, Some(contract_id)) => {
                Some(services::contract_delete_transaction_body::Obtainers::TransferContractId(
                    contract_id.to_protobuf(),
                ))
            }

            _ => None,
        };

        services::transaction_body::Data::ContractDeleteInstance(
            services::ContractDeleteTransactionBody {
                contract_id: delete_contract_id,
                permanent_removal: false,
                obtainers,
            },
        )
    }
}

impl FromProtobuf<services::ContractDeleteTransactionBody> for ContractDeleteTransactionData {
    fn from_protobuf(pb: services::ContractDeleteTransactionBody) -> crate::Result<Self> {
        use services::contract_delete_transaction_body::Obtainers;

        let (transfer_account_id, transfer_contract_id) = match pb.obtainers {
            Some(Obtainers::TransferAccountId(it)) => (Some(AccountId::from_protobuf(it)?), None),
            Some(Obtainers::TransferContractId(it)) => (None, Some(ContractId::from_protobuf(it)?)),
            None => (None, None),
        };

        Ok(Self {
            contract_id: Option::from_protobuf(pb.contract_id)?,
            transfer_account_id,
            transfer_contract_id,
        })
    }
}

impl From<ContractDeleteTransactionData> for AnyTransactionData {
    fn from(transaction: ContractDeleteTransactionData) -> Self {
        Self::ContractDelete(transaction)
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "ffi")]
    mod ffi {
        use assert_matches::assert_matches;

        use crate::contract::ContractDeleteTransaction;
        use crate::transaction::{
            AnyTransaction,
            AnyTransactionData,
        };
        use crate::{
            AccountId,
            ContractId,
        };

        // language=JSON
        const CONTRACT_DELETE_TRANSACTION_JSON: &str = r#"{
  "$type": "contractDelete",
  "contractId": "0.0.1001",
  "transferAccountId": "0.0.1002",
  "transferContractId": "0.0.1003"
}"#;

        #[test]
        fn it_should_serialize() -> anyhow::Result<()> {
            let mut transaction = ContractDeleteTransaction::new();

            transaction
                .contract_id(ContractId::from(1001))
                .transfer_account_id(AccountId::from(1002))
                .transfer_contract_id(ContractId::from(1003));

            let transaction_json = serde_json::to_string_pretty(&transaction)?;

            assert_eq!(transaction_json, CONTRACT_DELETE_TRANSACTION_JSON);

            Ok(())
        }

        #[test]
        fn it_should_deserialize() -> anyhow::Result<()> {
            let transaction: AnyTransaction =
                serde_json::from_str(CONTRACT_DELETE_TRANSACTION_JSON)?;

            let data = assert_matches!(transaction.data(), AnyTransactionData::ContractDelete(transaction) => transaction);

            assert_eq!(data.contract_id.unwrap(), ContractId::from(1001));
            assert_eq!(data.transfer_contract_id.unwrap(), ContractId::from(1003));
            assert_eq!(data.transfer_account_id, Some(AccountId::from(1002)));

            Ok(())
        }
    }
}
