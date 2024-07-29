// Copyright (c) SimpleStaking, Viable Systems and Tezedge Contributors
// SPDX-License-Identifier: MIT

use std::{collections::HashSet, convert::TryFrom};

use strum::IntoEnumIterator;

use crypto::hash::{ContextHash, ProtocolHash};
use mavryk_api::environment::{default_networks, MavrykEnvironment, MavrykEnvironmentConfiguration};
use mavryk_api::ffi::{
    InitProtocolContextResult, MavrykRuntimeConfiguration, MavrykRuntimeLogLevel,
    MavrykStorageInitError,
};
use mavryk_context_api::{
    PatchContext, MavrykContextConfiguration, MavrykContextIrminStorageConfiguration,
    MavrykContextStorageConfiguration, MavrykContextTezEdgeStorageConfiguration,
    MavrykContextTezedgeOnDiskBackendOptions,
};
use mavryk_interop::apply_encoded_message;
use mavryk_protocol_ipc_messages::{InitProtocolContextParams, NodeMessage, ProtocolMessage};

mod common;

fn init_protocol_context(
    context_config: MavrykContextConfiguration,
    mavryk_env: &MavrykEnvironmentConfiguration,
) -> Result<InitProtocolContextResult, MavrykStorageInitError> {
    let cfg = InitProtocolContextParams {
        storage: context_config.storage,
        genesis: context_config.genesis.clone(),
        genesis_max_operations_ttl: mavryk_env
            .genesis_additional_data()
            .unwrap()
            .max_operations_ttl,
        protocol_overrides: context_config.protocol_overrides.clone(),
        commit_genesis: true,
        enable_testchain: false,
        readonly: false,
        patch_context: context_config.sandbox_json_patch_context,
        context_stats_db_path: None,
    };
    let result = apply_encoded_message(ProtocolMessage::InitProtocolContextCall(cfg)).unwrap();
    let result = expect_response!(InitProtocolContextResult, result);
    result.map_err(|e| {
        MavrykStorageInitError::InitializeError {
            message: format!("FFI 'init_protocol_context' failed! Initialization of Mavryk context failed, this storage is required, we can do nothing without that, reason: {:?}", e)
        }
    })
}

#[test]
fn test_init_empty_context_for_all_enviroment_expect_custom_nets() {
    // init runtime and turn on/off ocaml logging
    apply_encoded_message(ProtocolMessage::ChangeRuntimeConfigurationCall(
        MavrykRuntimeConfiguration {
            log_level: Some(MavrykRuntimeLogLevel::Info),
            log_enabled: common::is_ocaml_log_enabled(),
        },
    ))
    .unwrap();

    // prepare data
    let storage_data_dir = "init_storage_tests_01";

    let mut genesis_commit_hashes: Vec<ContextHash> = Vec::new();
    let mut protocol_hashes: HashSet<ProtocolHash> = HashSet::new();

    // run init storage for all nets
    let default_networks = default_networks();
    let mut environment_counter = 0;
    MavrykEnvironment::iter()
        .filter(|te| *te != MavrykEnvironment::Custom)
        .for_each(|net| {
            environment_counter += 1;

            let mavryk_env: &MavrykEnvironmentConfiguration = default_networks
                .get(&net)
                .unwrap_or_else(|| panic!("no mavryk environment configured for: {:?}", &net));

            let storage = MavrykContextStorageConfiguration::Both(
                MavrykContextIrminStorageConfiguration {
                    data_dir: common::prepare_empty_dir(storage_data_dir),
                },
                MavrykContextTezEdgeStorageConfiguration {
                    backend: mavryk_context_api::ContextKvStoreConfiguration::InMem(
                        MavrykContextTezedgeOnDiskBackendOptions {
                            base_path: storage_data_dir.to_string(),
                            startup_check: false,
                        },
                    ),
                    ipc_socket_path: None,
                },
            );
            let context_config = MavrykContextConfiguration {
                storage,
                genesis: mavryk_env.genesis.clone(),
                protocol_overrides: mavryk_env.protocol_overrides.clone(),
                commit_genesis: true,
                enable_testchain: false,
                readonly: false,
                sandbox_json_patch_context: None,
                context_stats_db_path: None,
            };

            match init_protocol_context(context_config, mavryk_env) {
                Err(e) => panic!(
                    "Failed to initialize storage for: {:?}, Reason: {:?}",
                    net, e
                ),
                Ok(init_info) => {
                    if let Some(commit_hash) = &init_info.genesis_commit_hash {
                        genesis_commit_hashes.push(commit_hash.clone());
                    }
                    init_info
                        .supported_protocol_hashes
                        .iter()
                        .for_each(|protocol_hash| {
                            protocol_hashes.insert(protocol_hash.clone());
                        });
                }
            }
        });

    // check result - we should have
    assert_eq!(environment_counter, genesis_commit_hashes.len());
    assert!(protocol_hashes.len() > 1);
}

#[test]
fn test_init_empty_context_for_custom_network() {
    // init runtime and turn on/off ocaml logging
    common::init_test_runtime();

    // prepare data
    let storage_data_dir = "init_storage_tests_04";

    let mut genesis_commit_hashes: Vec<ContextHash> = Vec::new();
    let mut protocol_hashes: HashSet<ProtocolHash> = HashSet::new();

    // run init storage for all nets
    let custom_network_json = r#"{
            "network": {
                "chain_name": "SANDBOXED_MAVRYK",
                "genesis": {
                  "block": "BLockGenesisGenesisGenesisGenesisGenesisf79b5d1CoW2",
                  "protocol": "PtYuensgYBb3G3x1hLLbCmcav8ue8Kyd2khADcL5LsT5R1hcXex",
                  "timestamp": "2018-06-30T16:07:32Z"
                },
                "sandboxed_chain_name": "SANDBOXED_MAVRYK",
                "default_bootstrap_peers": [],
                "genesis_parameters": {
                  "values": {
                    "genesis_pubkey": "edpkuJQjuxBndWiwNRFGndPaJATFVXsiDDyAfE4oHvUtu138w5LYRs"
                  }
                }
            }
        }"#;

    let net = MavrykEnvironment::Custom;
    let mavryk_env = MavrykEnvironmentConfiguration::try_from_json(custom_network_json).unwrap();

    let storage = MavrykContextStorageConfiguration::Both(
        MavrykContextIrminStorageConfiguration {
            data_dir: common::prepare_empty_dir(storage_data_dir),
        },
        MavrykContextTezEdgeStorageConfiguration {
            backend: mavryk_context_api::ContextKvStoreConfiguration::InMem(
                MavrykContextTezedgeOnDiskBackendOptions {
                    base_path: storage_data_dir.to_string(),
                    startup_check: false,
                },
            ),
            ipc_socket_path: None,
        },
    );
    let context_config = MavrykContextConfiguration {
        storage,
        genesis: mavryk_env.genesis.clone(),
        protocol_overrides: mavryk_env.protocol_overrides.clone(),
        commit_genesis: true,
        enable_testchain: false,
        readonly: false,
        sandbox_json_patch_context: None,
        context_stats_db_path: None,
    };

    match init_protocol_context(context_config, &mavryk_env) {
        Err(e) => panic!(
            "Failed to initialize storage for: {:?}, Reason: {:?}",
            net, e
        ),
        Ok(init_info) => {
            if let Some(commit_hash) = &init_info.genesis_commit_hash {
                genesis_commit_hashes.push(commit_hash.clone());
            }
            init_info
                .supported_protocol_hashes
                .iter()
                .for_each(|protocol_hash| {
                    protocol_hashes.insert(protocol_hash.clone());
                });
        }
    }
}

#[test]
fn test_init_empty_context_for_sandbox_with_patch_json() -> Result<(), anyhow::Error> {
    // init runtime and turn on/off ocaml logging
    common::init_test_runtime();

    // prepare data
    let storage_data_dir = "init_storage_tests_02";

    // run init storage for all nets
    let net = MavrykEnvironment::Sandbox;
    let default_networks = default_networks();
    let mavryk_env: &MavrykEnvironmentConfiguration = default_networks
        .get(&net)
        .unwrap_or_else(|| panic!("no mavryk environment configured for: {:?}", &net));

    let patch_context = PatchContext {
        key: String::from("sandbox_parameter"),
        json: String::from(
            r#" { "genesis_pubkey": "edpkuSLWfVU1Vq7Jg9FucPyKmma6otcMHac9zG4oU1KMHSTBpJuGQ2"} "#,
        ),
    };

    let storage = MavrykContextStorageConfiguration::Both(
        MavrykContextIrminStorageConfiguration {
            data_dir: common::prepare_empty_dir(storage_data_dir),
        },
        MavrykContextTezEdgeStorageConfiguration {
            backend: mavryk_context_api::ContextKvStoreConfiguration::InMem(
                MavrykContextTezedgeOnDiskBackendOptions {
                    base_path: storage_data_dir.to_string(),
                    startup_check: false,
                },
            ),
            ipc_socket_path: None,
        },
    );
    let context_config = MavrykContextConfiguration {
        storage,
        genesis: mavryk_env.genesis.clone(),
        protocol_overrides: mavryk_env.protocol_overrides.clone(),
        commit_genesis: true,
        enable_testchain: false,
        readonly: false,
        sandbox_json_patch_context: Some(patch_context),
        context_stats_db_path: None,
    };

    match init_protocol_context(context_config, mavryk_env) {
        Err(e) => panic!(
            "Failed to initialize storage for: {:?}, Reason: {:?}",
            net, e
        ),
        Ok(init_info) => {
            if let Some(commit_hash) = &init_info.genesis_commit_hash {
                assert_eq!(
                    *commit_hash,
                    ContextHash::try_from("CoVBYdAGWBoDTkiVXJEGX6FQvDN1oGCPJu8STMvaTYdeh7N3KGTz")?
                )
            } else {
                panic!("Expected some context hash")
            }
        }
    }

    Ok(())
}

#[test]
fn test_init_empty_context_for_sandbox_without_patch_json() -> Result<(), anyhow::Error> {
    // init runtime and turn on/off ocaml logging
    common::init_test_runtime();

    // prepare data
    let storage_data_dir = "init_storage_tests_03";

    // run init storage for all nets
    let net = MavrykEnvironment::Sandbox;
    let default_networks = default_networks();
    let mavryk_env: &MavrykEnvironmentConfiguration = default_networks
        .get(&net)
        .unwrap_or_else(|| panic!("no mavryk environment configured for: {:?}", &net));

    let storage = MavrykContextStorageConfiguration::Both(
        MavrykContextIrminStorageConfiguration {
            data_dir: common::prepare_empty_dir(storage_data_dir),
        },
        MavrykContextTezEdgeStorageConfiguration {
            backend: mavryk_context_api::ContextKvStoreConfiguration::InMem(
                MavrykContextTezedgeOnDiskBackendOptions {
                    base_path: storage_data_dir.to_string(),
                    startup_check: false,
                },
            ),
            ipc_socket_path: None,
        },
    );
    let context_config = MavrykContextConfiguration {
        storage,
        genesis: mavryk_env.genesis.clone(),
        protocol_overrides: mavryk_env.protocol_overrides.clone(),
        commit_genesis: true,
        enable_testchain: false,
        readonly: false,
        sandbox_json_patch_context: None,
        context_stats_db_path: None,
    };

    match init_protocol_context(context_config, mavryk_env) {
        Err(e) => panic!(
            "Failed to initialize storage for: {:?}, Reason: {:?}",
            net, e
        ),
        Ok(init_info) => {
            if let Some(commit_hash) = &init_info.genesis_commit_hash {
                assert_eq!(
                    *commit_hash,
                    ContextHash::try_from("CoVewPVcrKctWXSbrRgoGD6NmkdbDhmTFk5oi1FZpEcRT3bmKxdQ")?
                )
            } else {
                panic!("Expected some context hash")
            }
        }
    }

    Ok(())
}
