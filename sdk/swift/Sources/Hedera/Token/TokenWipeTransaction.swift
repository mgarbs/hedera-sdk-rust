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

/// Wipes the provided amount of tokens from the specified account.
public final class TokenWipeTransaction: Transaction {
    /// Create a new `TokenWipeTransaction`.
    public init(
        tokenId: TokenId? = nil,
        amount: UInt64 = 0,
        serials: [UInt64] = []
    ) {
        self.tokenId = tokenId
        self.amount = amount
        self.serials = serials

        super.init()
    }

    public required init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)

        tokenId = try container.decodeIfPresent(.tokenId)
        amount = try container.decodeIfPresent(.amount) ?? 0
        serials = try container.decodeIfPresent(.serials) ?? []

        try super.init(from: decoder)
    }

    /// The token for which to wipe tokens.
    public var tokenId: TokenId? {
        willSet {
            ensureNotFrozen()
        }
    }

    /// Sets the token for which to wipe tokens.
    @discardableResult
    public func tokenId(_ tokenId: TokenId) -> Self {
        self.tokenId = tokenId

        return self
    }

    /// The account to be wiped.
    public var accountId: AccountId? {
        willSet {
            ensureNotFrozen()
        }
    }

    /// Sets the account to be wiped.
    @discardableResult
    public func accountId(_ accountId: AccountId) -> Self {
        self.accountId = accountId

        return self
    }

    /// The amount of a fungible token to wipe from the specified account.
    public var amount: UInt64 {
        willSet {
            ensureNotFrozen()
        }
    }

    //// Sets the amount of a fungible token to wipe from the specified account.
    @discardableResult
    public func amount(_ amount: UInt64) -> Self {
        self.amount = amount

        return self
    }

    /// The serial numbers of a non-fungible token to wipe from the specified account.
    public var serials: [UInt64] {
        willSet {
            ensureNotFrozen()
        }
    }

    /// Sets the serial numbers of a non-fungible token to wipe from the specified account.
    @discardableResult
    public func serials(_ serials: [UInt64]) -> Self {
        self.serials = serials

        return self
    }

    /// Add a serial number of a non-fungible token to wipe from the specified account
    @discardableResult
    public func addSerial(_ serial: UInt64) -> Self {
        self.serials.append(serial)

        return self
    }

    private enum CodingKeys: String, CodingKey {
        case tokenId
        case amount
        case serials
    }

    public override func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)

        try container.encode(tokenId, forKey: .tokenId)
        try container.encode(amount, forKey: .amount)
        try container.encode(serials, forKey: .serials)

        try super.encode(to: encoder)
    }

    internal override func validateChecksums(on ledgerId: LedgerId) throws {
        try tokenId?.validateChecksums(on: ledgerId)
        try super.validateChecksums(on: ledgerId)
    }
}
