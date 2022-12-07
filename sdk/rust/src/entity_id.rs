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

use std::fmt::{
    self,
    Debug,
    Display,
    Formatter,
};
use std::str::FromStr;

use itertools::Itertools;
use tinystr::TinyAsciiStr;

use crate::evm_address::EvmAddress;
use crate::{
    Client,
    Error,
    LedgerId,
};

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub struct Checksum(TinyAsciiStr<5>);

impl Checksum {
    fn from_bytes(bytes: [u8; 5]) -> Checksum {
        Checksum(TinyAsciiStr::from_bytes(&bytes).unwrap())
    }
}

impl FromStr for Checksum {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse()
            .map(|tiny_s| Checksum(tiny_s))
            .map_err(|_| Error::basic_parse("Expected checksum to be exactly 5 characters"))
    }
}

impl Display for Checksum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl Debug for Checksum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "\"{self}\"")
    }
}

pub trait AutoValidateChecksum {
    fn validate_checksum_for_ledger_id(&self, ledger_id: &LedgerId) -> Result<(), Error>;
}

impl<ID> AutoValidateChecksum for Option<ID>
where
    ID: AutoValidateChecksum,
{
    fn validate_checksum_for_ledger_id(&self, ledger_id: &LedgerId) -> Result<(), Error> {
        self.as_ref().map_or(Ok(()), |id| id.validate_checksum_for_ledger_id(ledger_id))
    }
}

/// The ID of an entity on the Hedera network.
#[derive(Hash, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "ffi", derive(serde_with::SerializeDisplay, serde_with::DeserializeFromStr))]
pub struct EntityId {
    /// A non-negative number identifying the shard containing this entity.
    pub shard: u64,

    /// A non-negative number identifying the realm within the shard containing this entity.
    pub realm: u64,

    /// A non-negative number identifying the entity within the realm containing this entity.
    pub num: u64,

    /// A checksum if the entity ID was read from a user inputted string which inclueded a checksum
    pub checksum: Option<Checksum>,
}

impl EntityId {
    pub(crate) fn from_solidity_address(address: &str) -> crate::Result<Self> {
        EvmAddress::from_str(address).map(Self::from)
    }

    pub(crate) fn to_solidity_address(self) -> crate::Result<String> {
        EvmAddress::try_from(self).map(|it| it.to_string())
    }

    pub(crate) fn generate_checksum(entity_id_string: &String, ledger_id: &LedgerId) -> Checksum {
        const P3: usize = 26 * 26 * 26; // 3 digits in base 26
        const P5: usize = 26 * 26 * 26 * 26 * 26; // 5 digits in base 26
        const M: usize = 1_000_003; // min prime greater than a million. Used for the final permutation.
        const W: usize = 31; // Sum s of digit values weights them by powers of W. Should be coprime to P5.

        let h = [ledger_id.to_bytes(), vec![0u8; 6]].concat();

        // Digits with 10 for ".", so if addr == "0.0.123" then d == [0, 10, 0, 10, 1, 2, 3]
        let d = entity_id_string.chars().map(|c| {
            if c == '.' {
                10_usize
            } else {
                c.to_digit(10).unwrap() as usize
            }
        });

        let mut s = 0; // Weighted sum of all positions (mod P3)
        let mut s0 = 0; // Sum of even positions (mod 11)
        let mut s1 = 0; // Sum of odd positions (mod 11)
        for (i, digit) in d.enumerate() {
            s = (W * s + digit) % P3;
            if i % 2 == 0 {
                s0 = (s0 + digit) % 11;
            } else {
                s1 = (s1 + digit) % 11;
            }
        }

        let mut sh = 0; // Hash of the ledger ID
        for b in h {
            sh = (W * sh + (b as usize)) % P5;
        }

        // The checksum, as a single number
        let mut c = ((((entity_id_string.len() % 5) * 11 + s0) * 11 + s1) * P3 + s + sh) % P5;
        c = (c * M) % P5;

        let mut answer = [0_u8; 5];
        for i in (0..5).rev() {
            answer[i] = b'a' + ((c % 26) as u8);
            c /= 26;
        }

        Checksum::from_bytes(answer)
    }

    pub(crate) async fn validate_checksum(
        shard: u64,
        realm: u64,
        num: u64,
        checksum: &Option<Checksum>,
        client: &Client,
    ) -> Result<(), Error> {
        if let Some(present_checksum) = checksum {
            if let Some(ledger_id) = client.ledger_id().await {
                Self::validate_checksum_internal(shard, realm, num, present_checksum, &ledger_id)
            } else {
                Err(Error::CannotPerformTaskWithoutLedgerId { task: "validate checksum" })
            }
        } else {
            Ok(())
        }
    }

    pub(crate) fn validate_checksum_for_ledger_id(
        shard: u64,
        realm: u64,
        num: u64,
        checksum: &Option<Checksum>,
        ledger_id: &LedgerId,
    ) -> Result<(), Error> {
        if let Some(present_checksum) = checksum {
            Self::validate_checksum_internal(shard, realm, num, present_checksum, ledger_id)
        } else {
            Ok(())
        }
    }

    fn validate_checksum_internal(
        shard: u64,
        realm: u64,
        num: u64,
        present_checksum: &Checksum,
        ledger_id: &LedgerId,
    ) -> Result<(), Error> {
        let expected_checksum =
            Self::generate_checksum(&format!("{}.{}.{}", shard, realm, num), ledger_id);
        if present_checksum != &expected_checksum {
            Err(Error::BadEntityId {
                shard,
                realm,
                num,
                present_checksum: present_checksum.clone(),
                expected_checksum,
            })
        } else {
            Ok(())
        }
    }

    pub(crate) async fn to_string_with_checksum(
        entity_id_string: String,
        client: &Client,
    ) -> Result<String, Error> {
        if let Some(ledger_id) = client.ledger_id().await {
            Ok(format!(
                "{}-{}",
                entity_id_string,
                Self::generate_checksum(&entity_id_string, &ledger_id)
            ))
        } else {
            Err(Error::CannotPerformTaskWithoutLedgerId { task: "derive checksum for entity ID" })
        }
    }
}

impl Debug for EntityId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "\"{self}\"")
    }
}

impl Display for EntityId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.shard, self.realm, self.num)
    }
}

impl From<u64> for EntityId {
    fn from(num: u64) -> Self {
        Self { num, shard: 0, realm: 0, checksum: None }
    }
}

impl FromStr for EntityId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let from_nums_and_checksum = |nums: &str, checksum| {
            let parts: Vec<u64> =
                nums.splitn(3, '.').map(u64::from_str).try_collect().map_err(Error::basic_parse)?;
            match *parts.as_slice() {
                [num] => Ok(Self::from(num)),
                [shard, realm, num] => Ok(Self { shard, realm, num, checksum }),
                _ => Err(Error::basic_parse("expecting <shard>.<realm>.<num> (ex. `0.0.1001`)")),
            }
        };

        if let Some((nums, raw_checksum)) = s.split_once('-') {
            from_nums_and_checksum(nums, Some(Checksum::from_str(raw_checksum)?))
        } else {
            from_nums_and_checksum(s, None)
        }
    }
}
