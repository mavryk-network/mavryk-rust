// Copyright (c) SimpleStaking, Viable Systems and Tezedge Contributors
// SPDX-License-Identifier: MIT

use std::sync::Arc;

use async_ipc::IpcError;
use crypto::hash::ContextHash;
use mavryk_context_api::{ContextKeyOwned, ContextValue, StringTreeObject};
use mavryk_protocol_ipc_client::{ProtocolRunnerApi, ProtocolServiceError};
use thiserror::Error;

#[derive(Clone)]
pub struct TezedgeContextClient {
    mavryk_protocol_api: Arc<ProtocolRunnerApi>,
}

#[derive(Debug, Error)]
pub enum TezedgeContextClientError {
    #[error("Protocol service error: {reason:?}")]
    ProtocolServiceError {
        #[from]
        reason: ProtocolServiceError,
    },
    #[error("IPC Error: {reason}")]
    IpcError {
        #[from]
        reason: IpcError,
    },
}

impl TezedgeContextClient {
    pub fn new(mavryk_protocol_api: Arc<ProtocolRunnerApi>) -> Self {
        Self { mavryk_protocol_api }
    }

    pub async fn get_key_from_history(
        &self,
        context_hash: &ContextHash,
        key: ContextKeyOwned,
    ) -> Result<Option<ContextValue>, TezedgeContextClientError> {
        Ok(self
            .mavryk_protocol_api
            .readable_connection()
            .await?
            .get_context_key_from_history(context_hash, key)
            .await?)
    }

    pub async fn get_key_values_by_prefix(
        &self,
        context_hash: &ContextHash,
        prefix: ContextKeyOwned,
    ) -> Result<Option<Vec<(ContextKeyOwned, ContextValue)>>, TezedgeContextClientError> {
        Ok(self
            .mavryk_protocol_api
            .readable_connection()
            .await?
            .get_context_key_values_by_prefix(context_hash, prefix)
            .await?)
    }

    pub async fn get_context_tree_by_prefix(
        &self,
        context_hash: &ContextHash,
        prefix: ContextKeyOwned,
        depth: Option<usize>,
    ) -> Result<StringTreeObject, TezedgeContextClientError> {
        Ok(self
            .mavryk_protocol_api
            .readable_connection()
            .await?
            .get_context_tree_by_prefix(context_hash, prefix, depth)
            .await?)
    }
}
