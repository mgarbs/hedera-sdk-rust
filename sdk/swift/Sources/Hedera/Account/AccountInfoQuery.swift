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

/// Get all the information about an account, including the balance.
///
/// This does not get the list of account records.
///
public final class AccountInfoQuery: Query<AccountInfo> {
    /// Create a new `AccountInfoQuery`.
    public init(
        accountId: AccountId? = nil
    ) {
        self.accountId = accountId
    }

    /// The account ID for which information is requested.
    public var accountId: AccountId?

    /// Sets the account ID for which information is requested.
    @discardableResult
    public func accountId(_ accountId: AccountId) -> Self {
        self.accountId = accountId

        return self
    }

    private enum CodingKeys: String, CodingKey {
        case accountId
    }

    public override func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)

        try container.encodeIfPresent(accountId, forKey: .accountId)

        try super.encode(to: encoder)
    }

    internal override func validateChecksums(on ledgerId: LedgerId) throws {
        try accountId?.validateChecksums(on: ledgerId)
        try super.validateChecksums(on: ledgerId)
    }
}
