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

import XCTest

@testable import Hedera

private let parsedNftId = NftId(tokenId: TokenId(shard: 1415, realm: 314, num: 123), serial: 456)

public final class NftIdTests: XCTestCase {
    public func testParseSlashFormat() {

        let actualNftId: NftId = "1415.314.123/456"

        XCTAssertEqual(parsedNftId, actualNftId)
    }

    public func testParseAtFormat() {
        let actualNftId: NftId = "1415.314.123@456"

        XCTAssertEqual(parsedNftId, actualNftId)
    }
}
