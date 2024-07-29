// Copyright (c) SimpleStaking, Viable Systems and Tezedge Contributors
// SPDX-License-Identifier: MIT
#![forbid(unsafe_code)]

use honggfuzz::fuzz;
use log::debug;

use mavryk_messages::p2p::binary_message::BinaryRead;
use mavryk_messages::p2p::encoding::prelude::*;

fn main() {
    loop {
        fuzz!(|data: &[u8]| {
            if let Err(e) = BlockHeaderMessage::from_bytes(data) {
                debug!(
                    "BlockHeaderMessage::from_bytes produced error for input: {:?}\nError:\n{:?}",
                    data, e
                );
            }
        });
    }
}
