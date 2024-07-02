// SPDX-FileCopyrightText: 2022-2023 TriliTech <contact@trili.tech>
//
// SPDX-License-Identifier: MIT

//! Hash of Layer1 contract ids.

use std::fmt::Display;
use mavryk_data_encoding::enc::BinWriter;
use mavryk_data_encoding::encoding::HasEncoding;
use mavryk_data_encoding::nom::NomReader;

use crate::base58::{FromBase58Check, FromBase58CheckError};
use crate::hash::{
    ContractMv1Hash, ContractMv2Hash, ContractMv3Hash, ContractMv4Hash, Hash, HashTrait, HashType,
};

/// Hash of Layer1 contract ids.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, HasEncoding, BinWriter, NomReader)]
pub enum PublicKeyHash {
    /// Mv1-contract
    Ed25519(ContractMv1Hash),
    /// Mv2-contract
    Secp256k1(ContractMv2Hash),
    /// Mv3-contract
    P256(ContractMv3Hash),
    /// Mv4-contract
    Bls(ContractMv4Hash),
}

impl Display for PublicKeyHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ed25519(mv1) => write!(f, "{}", mv1),
            Self::Secp256k1(mv2) => write!(f, "{}", mv2),
            Self::P256(mv3) => write!(f, "{}", mv3),
            Self::Bls(mv4) => write!(f, "{}", mv4),
        }
    }
}

impl PublicKeyHash {
    /// Conversion from base58-encoding string (with prefix).
    pub fn from_b58check(data: &str) -> Result<Self, FromBase58CheckError> {
        let bytes = data.from_base58check()?;
        match bytes {
            _ if bytes.starts_with(HashType::ContractMv1Hash.base58check_prefix()) => Ok(
                PublicKeyHash::Ed25519(ContractMv1Hash::from_b58check(data)?),
            ),
            _ if bytes.starts_with(HashType::ContractMv2Hash.base58check_prefix()) => Ok(
                PublicKeyHash::Secp256k1(ContractMv2Hash::from_b58check(data)?),
            ),
            _ if bytes.starts_with(HashType::ContractMv3Hash.base58check_prefix()) => {
                Ok(PublicKeyHash::P256(ContractMv3Hash::from_b58check(data)?))
            }
            _ if bytes.starts_with(HashType::ContractMv4Hash.base58check_prefix()) => {
                Ok(PublicKeyHash::Bls(ContractMv4Hash::from_b58check(data)?))
            }
            _ => Err(FromBase58CheckError::InvalidBase58),
        }
    }

    /// Conversion to base58-encoding string (with prefix).
    pub fn to_b58check(&self) -> String {
        match self {
            Self::Ed25519(mv1) => mv1.to_b58check(),
            Self::Secp256k1(mv2) => mv2.to_b58check(),
            Self::P256(mv3) => mv3.to_b58check(),
            Self::Bls(mv4) => mv4.to_b58check(),
        }
    }
}

impl From<PublicKeyHash> for Hash {
    fn from(pkh: PublicKeyHash) -> Self {
        match pkh {
            PublicKeyHash::Ed25519(mv1) => mv1.into(),
            PublicKeyHash::Secp256k1(mv2) => mv2.into(),
            PublicKeyHash::P256(mv3) => mv3.into(),
            PublicKeyHash::Bls(mv4) => mv4.into(),
        }
    }
}

impl TryFrom<&str> for PublicKeyHash {
    type Error = FromBase58CheckError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_b58check(value)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn mv1_b58check() {
        let mv1 = "mv1JsodwJE1jQfWQp4Xz2SKX6DksPSKyL7c2";

        let pkh = PublicKeyHash::from_b58check(mv1);

        assert!(matches!(pkh, Ok(PublicKeyHash::Ed25519(_))));

        let mv1_from_pkh = pkh.unwrap().to_b58check();

        assert_eq!(mv1, &mv1_from_pkh);
    }

    #[test]
    fn mv2_b58check() {
        let mv2 = "mv2PRSVwssq2vDiaJd8iUeVzQLrbDKWYxHRJ";

        let pkh = PublicKeyHash::from_b58check(mv2);

        assert!(matches!(pkh, Ok(PublicKeyHash::Secp256k1(_))));

        let mv2_from_pkh = pkh.unwrap().to_b58check();

        assert_eq!(mv2, &mv2_from_pkh);
    }

    #[test]
    fn mv3_b58check() {
        let mv3 = "mv3UBfJ2mvssa9VjrAPoPmQ8p7qLHPzXpUqS";

        let pkh = PublicKeyHash::from_b58check(mv3);

        assert!(matches!(pkh, Ok(PublicKeyHash::P256(_))));

        let mv3_from_pkh = pkh.unwrap().to_b58check();

        assert_eq!(mv3, &mv3_from_pkh);
    }

    #[test]
    fn mv4_b58check() {
        let mv4 = "mv4jRy7b4AieC8vodKYxvswdhENWC6hi5wey";

        let pkh = PublicKeyHash::from_b58check(mv4);

        assert!(matches!(pkh, Ok(PublicKeyHash::Bls(_))));

        let mv4_from_pkh = pkh.unwrap().to_b58check();

        assert_eq!(mv4, &mv4_from_pkh);
    }

    #[test]
    fn mv1_encoding() {
        let mv1 = "mv18Cw7psUrAAPBpXYd9CtCpHg9EgjHP9KTe";

        let pkh = PublicKeyHash::from_b58check(mv1).expect("expected valid mv1 hash");

        let mut bin = Vec::new();
        pkh.bin_write(&mut bin).expect("serialization should work");

        let deserde_pkh = NomReader::nom_read(bin.as_slice())
            .expect("deserialization should work")
            .1;

        // Check tag encoding
        assert_eq!(0_u8, bin[0]);
        assert_eq!(pkh, deserde_pkh);
    }

    #[test]
    fn mv2_encoding() {
        let mv2 = "mv2YQvRHUEk6FZikCC1jVNbFzzGdHfW8DD42";

        let pkh = PublicKeyHash::from_b58check(mv2).expect("expected valid mv2 hash");

        let mut bin = Vec::new();
        pkh.bin_write(&mut bin).expect("serialization should work");

        let deserde_pkh = NomReader::nom_read(bin.as_slice())
            .expect("deserialization should work")
            .1;

        // Check tag encoding
        assert_eq!(1_u8, bin[0]);
        assert_eq!(pkh, deserde_pkh);
    }

    #[test]
    fn mv3_encoding() {
        let mv3 = "mv3AskP1ToMYF5m4WxQTvGwMcPSQ2cBvBz3H";

        let pkh = PublicKeyHash::from_b58check(mv3).expect("expected valid mv3 hash");

        let mut bin = Vec::new();
        pkh.bin_write(&mut bin).expect("serialization should work");

        let deserde_pkh = NomReader::nom_read(bin.as_slice())
            .expect("deserialization should work")
            .1;

        // Check tag encoding
        assert_eq!(2_u8, bin[0]);
        assert_eq!(pkh, deserde_pkh);
    }

    #[test]
    fn mv4_encoding() {
        let mv4 = "mv4jRy7b4AieC8vodKYxvswdhENWC6hi5wey";

        let pkh = PublicKeyHash::from_b58check(mv4).expect("expected valid mv4 hash");

        let mut bin = Vec::new();
        pkh.bin_write(&mut bin).expect("serialization should work");

        let deserde_pkh = NomReader::nom_read(bin.as_slice())
            .expect("deserialization should work")
            .1;

        // Check tag encoding
        assert_eq!(3_u8, bin[0]);
        assert_eq!(pkh, deserde_pkh);
    }
}
