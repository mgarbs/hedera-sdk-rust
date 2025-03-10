/*
 * ‌
 * Hedera Swift SDK
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

import CHedera
import Foundation

/// Response from `AccountInfoQuery`.
public final class AccountInfo: Codable {
    /// The account that is being referenced.
    public let accountId: AccountId

    /// The Contract Account ID comprising of both the contract instance and the cryptocurrency
    /// account owned by the contract instance, in the format used by Solidity.
    public let contractAccountId: String

    /// If true, then this account has been deleted, it will disappear when it expires, and all
    /// transactions for it will fail except the transaction to extend its expiration date.
    public let isDeleted: Bool

    /// The Account ID of the account to which this is proxy staked.
    ///
    /// If `proxy_account_id` is `None`, an invalid account, or an account that isn't a node,
    /// then this account is automatically proxy staked to a node chosen by the network,
    /// but without earning payments.
    ///
    /// If the `proxy_account_id` account refuses to accept proxy staking, or if it is not currently
    /// running a node, then it will behave as if `proxy_account_id` is `None`.
    @available(*, deprecated)
    public let proxyAccountId: AccountId?

    /// The total number of HBARs proxy staked to this account.
    public let proxyReceived: Hbar

    /// The key for the account, which must sign in order to transfer out, or to modify the
    /// account in any way other than extending its expiration date.
    public let key: Key

    /// Current balance of the referenced account.
    public let balance: Hbar

    /// The threshold amount for which an account record is created (and this account
    /// charged for them) for any send/withdraw transaction.
    @available(*, deprecated)
    public let sendRecordThreshold: Hbar

    /// The threshold amount for which an account record is created
    /// (and this account charged for them) for any transaction above this amount.
    @available(*, deprecated)
    public let receiveRecordThreshold: Hbar

    /// If true, no transaction can transfer to this account unless signed by
    /// this account's key.
    public let isReceiverSignatureRequired: Bool

    /// The TimeStamp time at which this account is set to expire.
    public let expirationTime: Timestamp?

    /// The duration for expiration time will extend every this many seconds.
    public let autoRenewPeriod: Duration?

    /// The memo associated with the account.
    public let accountMemo: String

    /// The number of NFTs owned by this account
    public let ownedNfts: UInt64

    /// The maximum number of tokens that an Account can be implicitly associated with.
    public let maxAutomaticTokenAssociations: UInt32

    /// The public key which aliases to this account.
    public let aliasKey: PublicKey?

    /// The ethereum transaction nonce associated with this account.
    public let ethereumNonce: UInt64

    /// The ledger ID the response was returned from.
    public let ledgerId: LedgerId

    /// Staking metadata for this account.
    public let staking: StakingInfo?

    public static func fromBytes(_ bytes: Data) throws -> Self {
        try fromJsonBytes(bytes)
    }

    public func toBytes() -> Data {
        // can't have `throws` because that's the wrong function signature.
        // swiftlint:disable force_try
        try! toJsonBytes()
    }
}

extension AccountInfo: ToFromJsonBytes {
    internal static var cFromBytes: FromJsonBytesFunc { hedera_account_info_from_bytes }
    internal static var cToBytes: ToJsonBytesFunc { hedera_account_info_to_bytes }
}
