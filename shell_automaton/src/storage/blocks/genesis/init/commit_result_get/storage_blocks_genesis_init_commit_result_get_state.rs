// Copyright (c) SimpleStaking, Viable Systems and Tezedge Contributors
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

use mavryk_api::ffi::CommitGenesisResult;
use mavryk_protocol_ipc_client::ProtocolServiceError;

use crate::protocol_runner::ProtocolRunnerToken;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum StorageBlocksGenesisInitCommitResultGetState {
    Init {},
    Pending { token: ProtocolRunnerToken },
    Error { error: ProtocolServiceError },
    Success { result: CommitGenesisResult },
}
