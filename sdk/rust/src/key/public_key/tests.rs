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

use std::str::FromStr;

use assert_matches::assert_matches;
use expect_test::expect;
use hex_literal::hex;

use crate::PublicKey;

#[test]
fn ed25519_from_str() {
    const PK: &str =
        "302a300506032b6570032100e0c8ec2758a5879ffac226a13c0c516b799e72e35141a0dd828f94d37988a4b7";

    let pk = PublicKey::from_str(PK).unwrap();

    assert_eq!(PK, &pk.to_string())
}

#[test]
fn ecdsa_from_str() {
    const PK: &str = "302f300906072a8648ce3d020103220002703a9370b0443be6ae7c507b0aec81a55e94e4a863b9655360bd65358caa6588";

    let pk = PublicKey::from_str(PK).unwrap();

    assert_eq!(PK, &pk.to_string());
}

#[track_caller]
fn pk_from_str_variants(key: &str) {
    // a very low tech solution, but it works!
    for case in 0..4 {
        let prefix = case & 1 == 0;
        let uppercase = (case >> 1) & 1 == 1;
        let prefix = if prefix { "0x" } else { "" };
        let pk = if uppercase { key.to_uppercase() } else { key.to_lowercase() };

        let pk = format!("{prefix}{pk}");
        let pk = PublicKey::from_str(&pk).unwrap();

        assert_eq!(key, &pk.to_string())
    }
}

#[test]
fn ed25519_from_str_variants() {
    pk_from_str_variants(
        "302a300506032b6570032100e0c8ec2758a5879ffac226a13c0c516b799e72e35141a0dd828f94d37988a4b7",
    );
}

#[test]
fn ecdsa_from_str_variants() {
    pk_from_str_variants("302f300906072a8648ce3d020103220002703a9370b0443be6ae7c507b0aec81a55e94e4a863b9655360bd65358caa6588");
}

#[test]
fn to_evm_address() {
    let key = PublicKey::from_str_ecdsa(
        "029469a657510f3bf199a0e29b21e11e7039d8883f3547d59c3568f9c89f704cbc",
    )
    .unwrap();
    let evm_address = key.to_evm_address().unwrap();
    assert_eq!(evm_address, "0x83e31378423cf9c51ec36b074b89e16f31560cbb");
}

#[test]
fn ed25519_verify() {
    let pk = PublicKey::from_str(
        "302a300506032b6570032100e0c8ec2758a5879ffac226a13c0c516b799e72e35141a0dd828f94d37988a4b7",
    )
    .unwrap();

    let signature = hex!(
        "9d04bfed7baa97c80d29a6ae48c0d896ce8463a7ea0c16197d55a563c73996ef"
        "062b2adf507f416c108422c0310fc6fb21886e11ce3de3e951d7a56049743f07"
    );

    pk.verify(b"hello, world", &signature).unwrap();
}

#[test]
fn ecdsa_verify() {
    let pk = PublicKey::from_str(
  "302f300906072a8648ce3d020103220002703a9370b0443be6ae7c507b0aec81a55e94e4a863b9655360bd65358caa6588"
  )
  .unwrap();

    // notice that this doesn't match other impls
    // this is to avoid signature malleability.
    // see: https://github.com/bitcoin/bips/blob/43da5dec5eaf0d8194baa66ba3dd976f923f9d07/bip-0032.mediawiki
    let signature = hex!(
        "f3a13a555f1f8cd6532716b8f388bd4e9d8ed0b252743e923114c0c6cbfe414c"
        "086e3717a6502c3edff6130d34df252fb94b6f662d0cd27e2110903320563851"
    );

    pk.verify(b"hello world", &signature).unwrap();
}

#[test]
fn ed25519_verify_bad_signature() {
    let pk = PublicKey::from_str(
        "302a300506032b6570032100e0c8ec2758a5879ffac226a13c0c516b799e72e35141a0dd828f94d37988a4b7",
    )
    .unwrap();

    let signature = hex!(
        "9d04bfed7baa97c80d29a6ae48c0d896ce8463a7ea0c16197d55a563c73996ef"
        "062b2adf507f416c108422c0310fc6fb21886e11ce3de3e951d7a56049743f00"
    );

    let err = assert_matches!(pk.verify(b"hello, world", &signature), Err(e) => e);

    expect![
        "failed to verify a signature: signature error: Verification equation was not satisfied"
    ]
    .assert_eq(&err.to_string());
}

#[test]
fn ecdsa_verify_bad_signature() {
    let pk = PublicKey::from_str(
  "302f300906072a8648ce3d020103220002703a9370b0443be6ae7c507b0aec81a55e94e4a863b9655360bd65358caa6588"
  )
  .unwrap();

    let signature = hex!(
        "f3a13a555f1f8cd6532716b8f388bd4e9d8ed0b252743e923114c0c6cbfe414c"
        "086e3717a6502c3edff6130d34df252fb94b6f662d0cd27e2110903320563850"
    );

    let err = assert_matches!(pk.verify(b"hello world", &signature), Err(e) => e);

    expect!["failed to verify a signature: signature error"].assert_eq(&err.to_string());
}

#[test]
fn ed25519_verify_error_ecdsa() {
    let pk = PublicKey::from_str(
        "302a300506032b6570032100e0c8ec2758a5879ffac226a13c0c516b799e72e35141a0dd828f94d37988a4b7",
    )
    .unwrap();

    let signature = hex!(
        "f3a13a555f1f8cd6532716b8f388bd4e9d8ed0b252743e923114c0c6cbfe414c"
        "086e3717a6502c3edff6130d34df252fb94b6f662d0cd27e2110903320563851"
    );

    let err = assert_matches!(pk.verify(b"hello, world", &signature), Err(e) => e);

    expect!["failed to verify a signature: signature error"].assert_eq(&err.to_string());
}

#[test]
fn ecdsa_verify_error_ed25519() {
    let pk = PublicKey::from_str(
  "302f300906072a8648ce3d020103220002703a9370b0443be6ae7c507b0aec81a55e94e4a863b9655360bd65358caa6588"
  )
  .unwrap();

    let signature = hex!(
        "9d04bfed7baa97c80d29a6ae48c0d896ce8463a7ea0c16197d55a563c73996ef"
        "062b2adf507f416c108422c0310fc6fb21886e11ce3de3e951d7a56049743f07"
    );

    let err = assert_matches!(pk.verify(b"hello world", &signature), Err(e) => e);

    expect!["failed to verify a signature: signature error"].assert_eq(&err.to_string());
}
