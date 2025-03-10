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

#[allow(clippy::module_inception)]
mod key;
mod key_list;
mod private_key;
mod public_key;

pub use key::Key;
pub use key_list::KeyList;
pub use private_key::PrivateKey;
pub use public_key::PublicKey;

#[derive(Copy, Clone)]
pub(crate) enum KeyKind {
    Ed25519,
    Ecdsa,
}
