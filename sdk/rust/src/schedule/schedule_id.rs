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

use hedera_proto::services;

use crate::entity_id::{
    Checksum,
    ValidateChecksums,
};
use crate::{
    Client,
    EntityId,
    Error,
    FromProtobuf,
    LedgerId,
    ToProtobuf,
};

/// The unique identifier for a scheduled transaction on Hedera.
#[derive(Hash, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "ffi", derive(serde_with::SerializeDisplay, serde_with::DeserializeFromStr))]
#[repr(C)]
pub struct ScheduleId {
    /// A non-negative number identifying the shard containing this scheduled transaction.
    pub shard: u64,

    /// A non-negative number identifying the realm within the shard containing this scheduled transaction.
    pub realm: u64,

    /// A non-negative number identifying the entity within the realm containing this scheduled transaction.
    pub num: u64,

    /// A checksum if the schedule ID was read from a user inputted string which inclueded a checksum
    pub checksum: Option<Checksum>,
}

impl ScheduleId {
    /// Create a new `AccountBalance` from protobuf-encoded `bytes`.
    ///
    /// # Errors
    /// - [`Error::FromProtobuf`](crate::Error::FromProtobuf) if decoding the bytes fails to produce a valid protobuf.
    /// - [`Error::FromProtobuf`](crate::Error::FromProtobuf) if decoding the protobuf fails.
    pub fn from_bytes(bytes: &[u8]) -> crate::Result<Self> {
        FromProtobuf::from_bytes(bytes)
    }

    /// Create a `ScheduleId` from a solidity address.
    pub fn from_solidity_address(address: &str) -> crate::Result<Self> {
        let EntityId { shard, realm, num, checksum } = EntityId::from_solidity_address(address)?;

        Ok(Self { shard, realm, num, checksum })
    }

    /// Convert `self` to a protobuf-encoded [`Vec<u8>`].
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        ToProtobuf::to_bytes(self)
    }

    /// Convert `self` into a solidity `address`
    pub fn to_solidity_address(&self) -> crate::Result<String> {
        EntityId { shard: self.shard, realm: self.realm, num: self.num, checksum: None }
            .to_solidity_address()
    }

    /// Convert `self` to a string with a valid checksum.
    pub async fn to_string_with_checksum(&self, client: &Client) -> Result<String, Error> {
        EntityId::to_string_with_checksum(self.to_string(), client).await
    }

    /// If this schedule ID was constructed from a user input string, it might include a checksum.
    ///
    /// This function will validate that the checksum is correct, returning an `Err()` result containing an
    /// [`Error::BadEntityId`](crate::Error::BadEntityId) if it's invalid, and a `Some(())` result if it is valid.
    ///
    /// If no checksum is present, validation will silently pass (the function will return `Some(())`)
    pub async fn validate_checksum(&self, client: &Client) -> Result<(), Error> {
        EntityId::validate_checksum(self.shard, self.realm, self.num, &self.checksum, client).await
    }
}

impl ValidateChecksums for ScheduleId {
    fn validate_checksums_for_ledger_id(&self, ledger_id: &LedgerId) -> Result<(), Error> {
        EntityId::validate_checksum_for_ledger_id(
            self.shard,
            self.realm,
            self.num,
            &self.checksum,
            ledger_id,
        )
    }
}

impl Debug for ScheduleId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "\"{self}\"")
    }
}

impl Display for ScheduleId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.shard, self.realm, self.num)
    }
}

impl FromProtobuf<services::ScheduleId> for ScheduleId {
    fn from_protobuf(pb: services::ScheduleId) -> crate::Result<Self> {
        Ok(Self {
            num: pb.schedule_num as u64,
            shard: pb.shard_num as u64,
            realm: pb.realm_num as u64,
            checksum: None,
        })
    }
}

impl ToProtobuf for ScheduleId {
    type Protobuf = services::ScheduleId;

    fn to_protobuf(&self) -> Self::Protobuf {
        services::ScheduleId {
            schedule_num: self.num as i64,
            realm_num: self.realm as i64,
            shard_num: self.shard as i64,
        }
    }
}

impl From<u64> for ScheduleId {
    fn from(num: u64) -> Self {
        Self { num, shard: 0, realm: 0, checksum: None }
    }
}

impl FromStr for ScheduleId {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        EntityId::from_str(s).map(Self::from)
    }
}

impl From<EntityId> for ScheduleId {
    fn from(value: EntityId) -> Self {
        let EntityId { shard, realm, num, checksum } = value;

        Self { shard, realm, num, checksum }
    }
}
