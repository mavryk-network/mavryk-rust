// Copyright (c) SimpleStaking, Viable Systems and Tezedge Contributors
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};
use mavryk_api::ffi::CommitGenesisResult;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum StorageBlocksGenesisInitCommitResultPutState {
    Init { result: CommitGenesisResult },
    Error {},
    Success {},
}
