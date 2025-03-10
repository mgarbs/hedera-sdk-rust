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

import Foundation

/// Change properties for the given topic.
///
/// Any null field is ignored (left unchanged).
///
public final class TopicUpdateTransaction: Transaction {
    /// Create a new `TopicUpdateTransaction` ready for configuration.
    public override init() {
        super.init()
    }

    public init(
        topicId: TopicId? = nil,
        expirationTime: Timestamp? = nil,
        topicMemo: String = "",
        adminKey: Key? = nil,
        submitKey: Key? = nil,
        autoRenewPeriod: Duration? = nil,
        autoRenewAccountId: AccountId? = nil
    ) {
        self.topicId = topicId
        self.expirationTime = expirationTime
        self.topicMemo = topicMemo
        self.adminKey = adminKey
        self.submitKey = submitKey
        self.autoRenewPeriod = autoRenewPeriod
        self.autoRenewAccountId = autoRenewAccountId

        super.init()
    }

    public required init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)

        topicId = try container.decodeIfPresent(.topicId)
        expirationTime = try container.decodeIfPresent(.expirationTime)
        topicMemo = try container.decodeIfPresent(.topicMemo) ?? ""
        adminKey = try container.decodeIfPresent(.adminKey)
        submitKey = try container.decodeIfPresent(.submitKey)
        autoRenewPeriod = try container.decodeIfPresent(.autoRenewPeriod)
        autoRenewAccountId = try container.decodeIfPresent(.autoRenewAccountId)

        try super.init(from: decoder)
    }

    /// The topic ID which is being updated in this transaction.
    public var topicId: TopicId? {
        willSet {
            ensureNotFrozen()
        }
    }

    /// Sets the topic ID which is being updated in this transaction.
    @discardableResult
    public func topicId(_ topicId: TopicId) -> Self {
        self.topicId = topicId

        return self
    }

    /// The new expiration time to extend to (ignored if equal to or before the current one).
    public var expirationTime: Timestamp? {
        willSet {
            ensureNotFrozen()
        }
    }

    /// Sets the new expiration time to extend to (ignored if equal to or before the current one).
    @discardableResult
    public func expirationTime(_ expirationTime: Timestamp) -> Self {
        self.expirationTime = expirationTime

        return self
    }

    /// Short publicly visible memo about the topic. No guarantee of uniqueness.
    public var topicMemo: String = "" {
        willSet {
            ensureNotFrozen()
        }
    }

    /// Sets the short publicly visible memo about the topic.
    @discardableResult
    public func topicMemo(_ topicMemo: String) -> Self {
        self.topicMemo = topicMemo

        return self
    }

    /// Access control for `TopicUpdateTransaction` and `TopicDeleteTransaction`.
    public var adminKey: Key? {
        willSet {
            ensureNotFrozen()
        }
    }

    /// Sets the access control for `TopicUpdateTransaction` and `TopicDeleteTransaction`.
    @discardableResult
    public func adminKey(_ adminKey: Key) -> Self {
        self.adminKey = adminKey

        return self
    }

    /// Access control for `TopicMessageSubmitTransaction`.
    public var submitKey: Key? {
        willSet {
            ensureNotFrozen()
        }
    }

    /// Sets the access control for `TopicMessageSubmitTransaction`.
    @discardableResult
    public func submitKey(_ submitKey: Key) -> Self {
        self.submitKey = submitKey

        return self
    }

    /// The initial lifetime of the topic and the amount of time to attempt to
    /// extend the topic's lifetime by automatically at the topic's expiration time, if
    /// the `autoRenewAccountId` is configured.
    public var autoRenewPeriod: Duration? {
        willSet {
            ensureNotFrozen()
        }
    }

    /// Sets the initial lifetime of the topic and the amount of time to attempt to
    /// extend the topic's lifetime by automatically at the topic's expiration time.
    @discardableResult
    public func autoRenewPeriod(_ autoRenewPeriod: Duration) -> Self {
        self.autoRenewPeriod = autoRenewPeriod

        return self
    }

    /// Account to be used at the topic's expiration time to extend the life of the topic.
    public var autoRenewAccountId: AccountId? {
        willSet {
            ensureNotFrozen()
        }
    }

    /// Sets the account to be used at the topic's expiration time to extend the life of the topic.
    @discardableResult
    public func autoRenewAccountId(_ autoRenewAccountId: AccountId) -> Self {
        self.autoRenewAccountId = autoRenewAccountId

        return self
    }

    private enum CodingKeys: String, CodingKey {
        case topicId
        case expirationTime
        case topicMemo
        case adminKey
        case submitKey
        case autoRenewPeriod
        case autoRenewAccountId
    }

    public override func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)

        try container.encode(topicId, forKey: .topicId)
        try container.encodeIfPresent(expirationTime, forKey: .expirationTime)
        try container.encodeIfPresent(topicMemo, forKey: .topicMemo)
        try container.encodeIfPresent(adminKey, forKey: .adminKey)
        try container.encodeIfPresent(submitKey, forKey: .submitKey)
        try container.encodeIfPresent(autoRenewPeriod, forKey: .autoRenewPeriod)
        try container.encodeIfPresent(autoRenewAccountId, forKey: .autoRenewAccountId)

        try super.encode(to: encoder)
    }

    internal override func validateChecksums(on ledgerId: LedgerId) throws {
        try topicId?.validateChecksums(on: ledgerId)
        try autoRenewAccountId?.validateChecksums(on: ledgerId)
        try super.validateChecksums(on: ledgerId)
    }
}
