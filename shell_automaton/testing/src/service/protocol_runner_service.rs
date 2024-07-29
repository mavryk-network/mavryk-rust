// Copyright (c) SimpleStaking, Viable Systems and Tezedge Contributors
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

use crypto::hash::ChainId;
use mavryk_api::environment::MavrykEnvironmentConfiguration;
use mavryk_api::ffi::{
    ApplyBlockRequest, BeginConstructionRequest, ComputePathRequest, PreapplyBlockRequest,
    ProtocolRpcRequest, MavrykRuntimeConfiguration, ValidateOperationRequest,
};
use mavryk_context_api::{PatchContext, MavrykContextStorageConfiguration};
use mavryk_messages::p2p::encoding::block_header::{BlockHeader, Level};
use mavryk_protocol_ipc_client::ProtocolServiceError;
use mavryk_protocol_ipc_messages::GenesisResultDataParams;

use shell_automaton::protocol_runner::ProtocolRunnerToken;
pub use shell_automaton::service::protocol_runner_service::{
    ProtocolRunnerResponse, ProtocolRunnerService,
};
use shell_automaton::service::service_async_channel::ResponseTryRecvError;

#[derive(Debug, Clone)]
pub struct ProtocolRunnerServiceDummy {
    connections: slab::Slab<()>,
}

impl ProtocolRunnerServiceDummy {
    pub fn new() -> Self {
        Self {
            connections: Default::default(),
        }
    }

    fn new_token(&mut self) -> ProtocolRunnerToken {
        ProtocolRunnerToken::new_unchecked(self.connections.insert(()))
    }
}

impl Default for ProtocolRunnerServiceDummy {
    fn default() -> Self {
        Self::new()
    }
}

impl ProtocolRunnerService for ProtocolRunnerServiceDummy {
    fn try_recv(&mut self) -> Result<ProtocolRunnerResponse, ResponseTryRecvError> {
        Err(ResponseTryRecvError::Empty)
    }

    fn spawn_server(&mut self) {}

    fn init_runtime(&mut self, _: MavrykRuntimeConfiguration) -> ProtocolRunnerToken {
        self.new_token()
    }

    fn init_context(
        &mut self,
        _: MavrykContextStorageConfiguration,
        _: &MavrykEnvironmentConfiguration,
        _: bool,
        _: bool,
        _: bool,
        _: Option<PatchContext>,
        _: Option<PathBuf>,
    ) -> Result<ProtocolRunnerToken, ProtocolServiceError> {
        Ok(self.new_token())
    }

    fn init_context_ipc_server(
        &mut self,
        _: MavrykContextStorageConfiguration,
    ) -> ProtocolRunnerToken {
        self.new_token()
    }

    fn genesis_commit_result_get_init(
        &mut self,
        _: GenesisResultDataParams,
    ) -> ProtocolRunnerToken {
        self.new_token()
    }

    fn preapply_block(&mut self, _req: PreapplyBlockRequest) -> ProtocolRunnerToken {
        self.new_token()
    }

    fn apply_block(&mut self, _: ApplyBlockRequest) {}

    fn begin_construction(&mut self, _: BeginConstructionRequest) -> ProtocolRunnerToken {
        self.new_token()
    }

    fn validate_operation(&mut self, _: ValidateOperationRequest) -> ProtocolRunnerToken {
        self.new_token()
    }

    fn get_context_raw_bytes(&mut self, _: ProtocolRpcRequest) -> ProtocolRunnerToken {
        self.new_token()
    }

    fn get_endorsing_rights(&mut self, _: ProtocolRpcRequest) -> ProtocolRunnerToken {
        self.new_token()
    }

    fn get_validators(
        &mut self,
        _chain_id: ChainId,
        _block_header: BlockHeader,
        _level: Level,
    ) -> ProtocolRunnerToken {
        self.new_token()
    }

    fn get_cycle_delegates(&mut self, _: ProtocolRpcRequest) -> ProtocolRunnerToken {
        self.new_token()
    }

    fn compute_operations_paths(&mut self, _: ComputePathRequest) -> ProtocolRunnerToken {
        self.new_token()
    }

    /// Notify status of protocol runner's and it's context initialization.
    fn notify_status(&mut self, _: bool) {}

    fn shutdown(&mut self) {}

    fn get_latest_context_hashes(&mut self, _: i64) -> ProtocolRunnerToken {
        self.new_token()
    }
}
