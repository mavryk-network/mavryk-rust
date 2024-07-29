// Copyright (c) SimpleStaking, Viable Systems and Tezedge Contributors
// SPDX-License-Identifier: MIT
#![forbid(unsafe_code)]
#![cfg_attr(feature = "fuzzing", feature(no_coverage))]

//! This crate provides serialization and deserialization functionality for the data types used by the Mavryk shell.

mod bit_utils;
pub mod types;

pub mod binary_reader;
pub mod binary_writer;

pub mod enc;
pub mod encoding;
pub mod nom;

#[cfg(feature = "fuzzing")]
pub mod fuzzing;
