// Copyright (c) SimpleStaking, Viable Systems and Tezedge Contributors
// SPDX-License-Identifier: MIT

#![allow(dead_code)]

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crypto::hash::ChainId;
use mavryk_api::environment::get_empty_operation_list_list_hash;
use mavryk_api::environment::GenesisAdditionalData;
use mavryk_api::environment::MavrykEnvironmentConfiguration;
use mavryk_api::ffi::InitProtocolContextResult;
use mavryk_api::ffi::MavrykRuntimeConfiguration;
use mavryk_api::ffi::MavrykRuntimeLogLevel;
use mavryk_context_api::MavrykContextIrminStorageConfiguration;
use mavryk_context_api::MavrykContextStorageConfiguration;
use mavryk_context_api::MavrykContextTezEdgeStorageConfiguration;
use mavryk_context_api::MavrykContextTezedgeOnDiskBackendOptions;
use mavryk_interop::apply_encoded_message;
use mavryk_messages::p2p::encoding::block_header::BlockHeader;
use mavryk_protocol_ipc_messages::InitProtocolContextParams;
use mavryk_protocol_ipc_messages::NodeMessage;
use mavryk_protocol_ipc_messages::ProtocolMessage;

#[allow(dead_code)]
pub fn prepare_empty_dir(dir_name: &str) -> String {
    let path = test_storage_dir_path(dir_name);
    if path.exists() {
        fs::remove_dir_all(&path)
            .unwrap_or_else(|_| panic!("Failed to delete directory: {:?}", &path));
    }
    fs::create_dir_all(&path).unwrap_or_else(|_| panic!("Failed to create directory: {:?}", &path));
    String::from(path.to_str().unwrap())
}

#[allow(dead_code)]
pub fn test_storage_dir_path(dir_name: &str) -> PathBuf {
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR is not defined");
    let path = Path::new(out_dir.as_str()).join(Path::new(dir_name));
    path
}

pub fn is_ocaml_log_enabled() -> bool {
    env::var("OCAML_LOG_ENABLED")
        .unwrap_or_else(|_| "false".to_string())
        .parse::<bool>()
        .unwrap()
}

#[allow(dead_code)]
pub fn init_test_runtime() {
    // init runtime and turn on/off ocaml logging
    apply_encoded_message(ProtocolMessage::ChangeRuntimeConfigurationCall(
        MavrykRuntimeConfiguration {
            log_level: Some(MavrykRuntimeLogLevel::Info),
            log_enabled: is_ocaml_log_enabled(),
        },
    ))
    .unwrap();
}

#[macro_export]
macro_rules! expect_response {
    ($id:ident, $result:ident) => {
        if let NodeMessage::$id(result) = $result {
            result
        } else {
            panic!(
                "Expected NodeMessage::{} response but got something else",
                stringify!($id)
            );
        }
    };
}

#[allow(dead_code)]
pub fn init_test_protocol_context(
    dir_name: &str,
    mavryk_env: MavrykEnvironmentConfiguration,
) -> (
    ChainId,
    BlockHeader,
    GenesisAdditionalData,
    InitProtocolContextResult,
) {
    // TODO: maybe accept storage configuration instead
    let storage = MavrykContextStorageConfiguration::Both(
        MavrykContextIrminStorageConfiguration {
            data_dir: prepare_empty_dir(dir_name),
        },
        MavrykContextTezEdgeStorageConfiguration {
            backend: mavryk_context_api::ContextKvStoreConfiguration::InMem(
                MavrykContextTezedgeOnDiskBackendOptions {
                    base_path: dir_name.to_string(),
                    startup_check: false,
                },
            ),
            ipc_socket_path: None,
        },
    );
    let context_config = InitProtocolContextParams {
        storage,
        genesis: mavryk_env.genesis.clone(),
        genesis_max_operations_ttl: mavryk_env
            .genesis_additional_data()
            .unwrap()
            .max_operations_ttl,
        protocol_overrides: mavryk_env.protocol_overrides.clone(),
        commit_genesis: true,
        enable_testchain: false,
        readonly: false,
        patch_context: mavryk_env.patch_context_genesis_parameters.clone(),
        context_stats_db_path: None,
    };
    let result =
        apply_encoded_message(ProtocolMessage::InitProtocolContextCall(context_config)).unwrap();
    let result = expect_response!(InitProtocolContextResult, result).unwrap();

    let genesis_commit_hash = match result.genesis_commit_hash.as_ref() {
        None => panic!("we needed commit_genesis and here should be result of it"),
        Some(cr) => cr.clone(),
    };

    (
        mavryk_env.main_chain_id().expect("invalid chain id"),
        mavryk_env
            .genesis_header(
                genesis_commit_hash,
                get_empty_operation_list_list_hash().unwrap(),
            )
            .expect("genesis header error"),
        mavryk_env
            .genesis_additional_data()
            .expect("protocol_hash error"),
        result,
    )
}
