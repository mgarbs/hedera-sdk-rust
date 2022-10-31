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

// todo: deduplicate these with `PrivateKey.swift`

private typealias UnsafeFromBytesFunc = @convention(c) (
    UnsafePointer<UInt8>?, Int, UnsafeMutablePointer<OpaquePointer?>?
) -> HederaError

/// A public key on the Hedera network.
public final class PublicKey: LosslessStringConvertible, ExpressibleByStringLiteral, Codable {
    internal let ptr: OpaquePointer

    // sadly, we can't avoid a leaky abstraction here.
    internal static func unsafeFromPtr(_ ptr: OpaquePointer) -> Self {
        Self.init(ptr)
    }

    private init(_ ptr: OpaquePointer) {
        self.ptr = ptr
    }

    private static func unsafeFromAnyBytes(_ bytes: Data, _ chederaCallback: UnsafeFromBytesFunc) throws -> Self {
        let ptr = try bytes.withUnsafeBytes { (pointer: UnsafeRawBufferPointer) in
            var key = OpaquePointer(bitPattern: 0)
            let err = chederaCallback(pointer.bindMemory(to: UInt8.self).baseAddress, pointer.count, &key)

            if err != HEDERA_ERROR_OK {
                throw HError(err)!
            }

            return key!
        }

        return Self(ptr)
    }

    public static func fromBytes(_ bytes: Data) throws -> Self {
        try unsafeFromAnyBytes(bytes, hedera_public_key_from_bytes)
    }

    public static func fromBytesEd25519(_ bytes: Data) throws -> Self {
        try unsafeFromAnyBytes(bytes, hedera_public_key_from_bytes_ed25519)
    }

    public static func fromBytesEcdsa(_ bytes: Data) throws -> Self {
        try unsafeFromAnyBytes(bytes, hedera_public_key_from_bytes_ed25519)
    }

    public static func fromBytesDer(_ bytes: Data) throws -> Self {
        try unsafeFromAnyBytes(bytes, hedera_public_key_from_bytes_ed25519)
    }

    public static func fromString(_ description: String) throws -> Self {
        var key = OpaquePointer(bitPattern: 0)
        let err = hedera_public_key_from_string(description, &key)

        if err != HEDERA_ERROR_OK {
            throw HError(err)!
        }

        return Self(key!)
    }

    public init?(_ description: String) {
        var key = OpaquePointer.init(bitPattern: 0)
        let err = hedera_public_key_from_string(description, &key)

        if err != HEDERA_ERROR_OK {
            return nil
        }

        ptr = key!
    }

    public required convenience init(stringLiteral value: StringLiteralType) {
        self.init(value)!
    }

    public static func fromStringDer(_ description: String) throws -> Self {
        var key = OpaquePointer(bitPattern: 0)
        let err = hedera_public_key_from_string_der(description, &key)

        if err != HEDERA_ERROR_OK {
            throw HError(err)!
        }

        return Self(key!)
    }

    public static func fromStringEd25519(_ description: String) throws -> Self {
        var key = OpaquePointer(bitPattern: 0)
        let err = hedera_public_key_from_string_ed25519(description, &key)

        if err != HEDERA_ERROR_OK {
            throw HError(err)!
        }

        return Self(key!)
    }

    public static func fromStringEcdsa(_ description: String) throws -> Self {
        var key = OpaquePointer(bitPattern: 0)
        let err = hedera_public_key_from_string_ecdsa(description, &key)

        if err != HEDERA_ERROR_OK {
            throw HError(err)!
        }

        return Self(key!)
    }

    public required convenience init(from decoder: Decoder) throws {
        self.init(try decoder.singleValueContainer().decode(String.self))!
    }

    public func toBytesDer() -> Data {
        var buf: UnsafeMutablePointer<UInt8>?
        let size = hedera_public_key_to_bytes_der(ptr, &buf)

        return Data(bytesNoCopy: buf!, count: size, deallocator: Data.unsafeCHederaBytesFree)
    }

    public func toBytes() -> Data {
        var buf: UnsafeMutablePointer<UInt8>?
        let size = hedera_public_key_to_bytes(ptr, &buf)

        return Data(bytesNoCopy: buf!, count: size, deallocator: Data.unsafeCHederaBytesFree)
    }

    public func toBytesRaw() -> Data {
        var buf: UnsafeMutablePointer<UInt8>?
        let size = hedera_public_key_to_bytes_raw(ptr, &buf)

        return Data(bytesNoCopy: buf!, count: size, deallocator: Data.unsafeCHederaBytesFree)
    }

    public var description: String {
        let descriptionBytes = hedera_public_key_to_string(ptr)
        return String(hString: descriptionBytes!)!
    }

    public func toString() -> String {
        description
    }

    public func toStringDer() -> String {
        let stringBytes = hedera_public_key_to_string_der(ptr)
        return String(hString: stringBytes!)!
    }

    public func toStringRaw() -> String {
        let stringBytes = hedera_public_key_to_string_raw(ptr)
        return String(hString: stringBytes!)!
    }

    public func toAccountId(shard: UInt64, realm: UInt64) -> AccountId {
        AccountId.init(shard: shard, realm: realm, alias: self)
    }

    public func isEd25519() -> Bool {
        hedera_public_key_is_ed25519(ptr)
    }

    public func isEcdsa() -> Bool {
        hedera_public_key_is_ecdsa(ptr)
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()

        try container.encode(String(describing: self))
    }

    deinit {
        hedera_public_key_free(ptr)
    }
}
