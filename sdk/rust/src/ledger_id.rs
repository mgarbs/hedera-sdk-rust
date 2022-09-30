use std::fmt::{
    self,
    Debug,
    Display,
    Formatter,
};
use std::str::FromStr;

use serde_with::{
    DeserializeFromStr,
    SerializeDisplay,
};

use crate::Error;

#[derive(Copy, Clone, Eq, PartialEq)]
enum Network {
    Mainnet,
    Testnet,
    Previewnet,
}

impl Network {
    const fn as_str(&self) -> &'static str {
        match self {
            Network::Mainnet => "mainnet",
            Network::Testnet => "testnet",
            Network::Previewnet => "previewnet",
        }
    }

    const fn as_bytes(&self) -> &'static [u8] {
        match self {
            Network::Mainnet => &[0],
            Network::Testnet => &[1],
            Network::Previewnet => &[2],
        }
    }
}

#[derive(Clone)]
enum LedgerIdData {
    Known(Network),
    Static(&'static [u8]),
    Other(Box<[u8]>),
}

#[derive(Clone, SerializeDisplay, DeserializeFromStr)]
pub struct LedgerId(LedgerIdData);

impl PartialEq for LedgerId {
    fn eq(&self, other: &Self) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl Eq for LedgerId {}

impl From<Network> for LedgerId {
    fn from(it: Network) -> Self {
        Self(LedgerIdData::Known(it))
    }
}

impl LedgerId {
    pub const MAINNET: LedgerId = LedgerId(LedgerIdData::Known(Network::Mainnet));
    pub const TESTNET: LedgerId = LedgerId(LedgerIdData::Known(Network::Testnet));
    pub const PREVIEWNET: LedgerId = LedgerId(LedgerIdData::Known(Network::Previewnet));

    #[must_use]
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        match &*bytes {
            [0] => Self::MAINNET,
            [1] => Self::TESTNET,
            [2] => Self::PREVIEWNET,
            _ => Self(LedgerIdData::Other(bytes.into_boxed_slice())),
        }
    }

    #[must_use]
    pub const fn from_static(bytes: &'static [u8]) -> Self {
        // this can be deduplicated with the previous nicely once `const_option_ext` is stable.
        match &*bytes {
            [0] => Self::MAINNET,
            [1] => Self::TESTNET,
            [2] => Self::PREVIEWNET,
            _ => Self(LedgerIdData::Static(bytes)),
        }
    }

    // can't match the constants because of `StructuralEq`
    #[must_use]
    pub const fn is_mainnet(&self) -> bool {
        matches!(self, &LedgerId(LedgerIdData::Known(Network::Mainnet)))
    }

    #[must_use]
    pub const fn is_testnet(&self) -> bool {
        matches!(self, &LedgerId(LedgerIdData::Known(Network::Testnet)))
    }

    #[must_use]
    pub const fn is_previewnet(&self) -> bool {
        matches!(self, &LedgerId(LedgerIdData::Known(Network::Previewnet)))
    }

    #[must_use]
    pub const fn is_known_network(&self) -> bool {
        matches!(self, Self(LedgerIdData::Known(_)))
    }

    #[must_use]
    pub const fn as_bytes(&self) -> &[u8] {
        match &self.0 {
            LedgerIdData::Known(network) => network.as_bytes(),
            LedgerIdData::Static(bytes) => bytes,
            LedgerIdData::Other(bytes) => &*bytes,
        }
    }

    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }
}

impl Debug for LedgerId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\"", self)
    }
}

impl Display for LedgerId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.0 {
            LedgerIdData::Known(net) => f.write_str(net.as_str()),
            _ => f.write_str(&hex::encode(self.as_bytes())),
        }
    }
}

impl FromStr for LedgerId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "mainnet" => Ok(Self::MAINNET),
            "testnet" => Ok(Self::TESTNET),
            "previewnet" => Ok(Self::PREVIEWNET),
            _ => hex::decode(s).map(Self::from_bytes).map_err(Error::basic_parse),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::LedgerId;

    const NETWORK1_BYTES: &[u8] = &[0x00, 0xff, 0x00, 0xff];
    const NETWORK1: LedgerId = LedgerId::from_static(NETWORK1_BYTES);

    #[test]
    fn to_string() {
        assert_eq!(LedgerId::MAINNET.to_string(), "mainnet");
        assert_eq!(LedgerId::TESTNET.to_string(), "testnet");
        assert_eq!(LedgerId::PREVIEWNET.to_string(), "previewnet");
        assert_eq!(NETWORK1.to_string(), "00ff00ff");
    }

    #[test]
    fn parse() {
        assert_eq!(LedgerId::from_str("mainnet").unwrap(), LedgerId::MAINNET);
        assert_eq!(LedgerId::from_str("testnet").unwrap(), LedgerId::TESTNET);
        assert_eq!(LedgerId::from_str("previewnet").unwrap(), LedgerId::PREVIEWNET);
        assert_eq!(LedgerId::from_str("00ff00ff").unwrap(), NETWORK1);
        assert_eq!(LedgerId::from_str("00FF00FF").unwrap(), NETWORK1);
    }

    #[test]
    fn as_bytes() {
        assert_eq!(NETWORK1.as_bytes(), NETWORK1_BYTES);
    }

    #[test]
    fn to_bytes() {
        assert_eq!(&*NETWORK1.to_bytes(), NETWORK1_BYTES);
    }
}
