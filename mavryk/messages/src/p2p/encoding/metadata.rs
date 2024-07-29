// Copyright (c) SimpleStaking, Viable Systems and Tezedge Contributors
// SPDX-License-Identifier: MIT

use std::fmt;

use getset::CopyGetters;
use serde::{Deserialize, Serialize};

use mavryk_encoding::enc::BinWriter;
use mavryk_encoding::encoding::HasEncoding;
use mavryk_encoding::nom::NomReader;

use crate::p2p::binary_message::SizeFromChunk;

#[cfg_attr(feature = "fuzzing", derive(fuzzcheck::DefaultMutator))]
#[derive(
    Serialize, Deserialize, CopyGetters, HasEncoding, NomReader, BinWriter, PartialEq, Clone,
)]
pub struct MetadataMessage {
    #[get_copy = "pub"]
    disable_mempool: bool,
    #[get_copy = "pub"]
    private_node: bool,
}

impl MetadataMessage {
    pub fn new(disable_mempool: bool, private_node: bool) -> Self {
        MetadataMessage {
            disable_mempool,
            private_node,
        }
    }
}

impl fmt::Debug for MetadataMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[disable_mempool: {}, private_node: {:?}]",
            self.disable_mempool, self.private_node
        )
    }
}

impl SizeFromChunk for MetadataMessage {
    fn size_from_chunk(
        _bytes: impl AsRef<[u8]>,
    ) -> Result<usize, mavryk_encoding::binary_reader::BinaryReaderError> {
        Ok(2)
    }
}
