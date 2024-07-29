// Copyright (c) SimpleStaking, Viable Systems and Tezedge Contributors
// SPDX-License-Identifier: MIT

use std::io;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{
    collections::HashMap,
    convert::{TryFrom, TryInto},
    env,
    ffi::OsString,
    fs,
};

use serde::{Deserialize, Serialize};
use slog::{debug, info, Logger};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use thiserror::Error;

use crypto::hash::{
    chain_id_from_block_hash, BlockHash, ChainId, ContextHash, OperationListListHash, ProtocolHash,
};
use crypto::{base58::FromBase58CheckError, blake2b::Blake2bError};
use mavryk_messages::p2p::encoding::prelude::{BlockHeader, BlockHeaderBuilder};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use crate::mavkit_config::MavkitConfig;
use mavryk_context_api::{GenesisChain, PatchContext, ProtocolOverrides};

pub const PROTOCOL_HASH_ZERO_BASE58_CHECK: &str =
    "PrihK96nBAFSxVL1GLJTVhu9YnzkMFiBeuJRPA8NwuZVZCE1L6i";

/// alternative to ocaml Operation_list_list_hash.empty
pub fn get_empty_operation_list_list_hash() -> Result<OperationListListHash, FromBase58CheckError> {
    OperationListListHash::try_from("LLoZS2LW3rEi7KYU4ouBQtorua37aWWCtpDmv1n2x3xoKi6sVXLWp")
}

/// Enum representing different Mavryk environment.
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq, Hash, EnumIter)]
pub enum MavrykEnvironment {
    Custom,
    Mainnet,
    Sandbox,
    Zeronet,
    Alphanet,
    Babylonnet,
    Carthagenet,
    Delphinet,
    Edonet,
    Edo2net,
    Florencenet,
    Granadanet,
    Hangzhounet,
    Ithacanet,
    Jakartanet,
}

impl MavrykEnvironment {
    pub fn possible_values() -> Vec<&'static str> {
        let mut possible_values = Vec::new();
        for sp in MavrykEnvironment::iter() {
            possible_values.extend(sp.supported_values());
        }
        possible_values
    }

    pub fn supported_values(&self) -> Vec<&'static str> {
        match self {
            MavrykEnvironment::Custom => vec!["custom"],
            MavrykEnvironment::Mainnet => vec!["mainnet"],
            MavrykEnvironment::Sandbox => vec!["sandbox"],
            MavrykEnvironment::Zeronet => vec!["zeronet"],
            MavrykEnvironment::Alphanet => vec!["alphanet"],
            MavrykEnvironment::Babylonnet => vec!["babylonnet", "babylon"],
            MavrykEnvironment::Carthagenet => vec!["carthagenet", "carthage"],
            MavrykEnvironment::Delphinet => vec!["delphinet", "delphi"],
            MavrykEnvironment::Edonet => vec!["edonet", "edo"],
            MavrykEnvironment::Edo2net => vec!["edo2net", "edo2"],
            MavrykEnvironment::Florencenet => vec!["florencenet", "florence"],
            MavrykEnvironment::Granadanet => vec!["granadanet", "granada"],
            MavrykEnvironment::Hangzhounet => vec!["hangzhounet", "hangzhou"],
            MavrykEnvironment::Ithacanet => vec!["ithacanet", "ithaca"],
            MavrykEnvironment::Jakartanet => vec!["jakartanet", "jakarta"],
        }
    }

    pub fn check_deprecated_network(&self) -> Option<String> {
        match self {
            MavrykEnvironment::Custom => None,
            MavrykEnvironment::Mainnet => None,
            MavrykEnvironment::Sandbox => None,
            MavrykEnvironment::Zeronet => Some(Self::deprecated_testnet_notice(
                MavrykEnvironment::Zeronet,
                vec![MavrykEnvironment::Delphinet, MavrykEnvironment::Edo2net],
            )),
            MavrykEnvironment::Alphanet => Some(Self::deprecated_testnet_notice(
                MavrykEnvironment::Alphanet,
                vec![MavrykEnvironment::Delphinet, MavrykEnvironment::Edo2net],
            )),
            MavrykEnvironment::Babylonnet => Some(Self::deprecated_testnet_notice(
                MavrykEnvironment::Babylonnet,
                vec![MavrykEnvironment::Delphinet, MavrykEnvironment::Edo2net],
            )),
            MavrykEnvironment::Carthagenet => Some(Self::deprecated_testnet_notice(
                MavrykEnvironment::Carthagenet,
                vec![MavrykEnvironment::Delphinet, MavrykEnvironment::Edo2net],
            )),
            MavrykEnvironment::Delphinet => None,
            MavrykEnvironment::Edonet => Some(Self::deprecated_net_notice(
                "EDONET",
                MavrykEnvironment::Edonet,
                MavrykEnvironment::Edo2net,
            )),
            MavrykEnvironment::Edo2net => None,
            MavrykEnvironment::Florencenet => None,
            MavrykEnvironment::Granadanet => None,
            MavrykEnvironment::Hangzhounet => None,
            MavrykEnvironment::Ithacanet => None,
            MavrykEnvironment::Jakartanet => None,
        }
    }

    // fn not_yet_fully_supported_testnet_notice(selected_network: MavrykEnvironment) -> String {
    //     let mut selected = selected_network.supported_values();
    //     selected.sort();
    //     format!(
    //         "\n\n\n\n////////////////////////////////////////// \
    //         \n//      !!! COOMING SOON TESTNET !!!      //\
    //         \n////////////////////////////////////////// \
    //         \n// Selected network: {:?} \
    //         \n// Is not fully supported yet, \
    //         \n// but will be soon. \
    //         \n// Possible problems: \
    //         \n// - no peers to connect \
    //         \n// - no data to download \
    //         \n// - no block application \
    //         \n// - RPCs missing \
    //         \n//////////////////////////////////////////\n\n\n\n",
    //         selected
    //     )
    // }

    fn deprecated_testnet_notice(
        selected_network: MavrykEnvironment,
        alternate_networks: Vec<MavrykEnvironment>,
    ) -> String {
        let mut selected = selected_network.supported_values();
        selected.sort_unstable();
        let mut alternate_networks = alternate_networks
            .iter()
            .flat_map(|rn| rn.supported_values())
            .collect::<Vec<_>>();
        alternate_networks.sort_unstable();
        format!(
            "\n\n\n\n////////////////////////////////////////// \
            \n//      !!! DEPRECATED TESTNET !!!      //\
            \n////////////////////////////////////////// \
            \n// Selected (deprecated) network: {:?} \
            \n// Use recommended network(s): {:?} \
            \n// Possible problems: \
            \n// - no peers to connect \
            \n// - no data to download \
            \n//////////////////////////////////////////\n\n\n\n",
            selected, alternate_networks
        )
    }

    fn deprecated_net_notice(
        title: &str,
        selected: MavrykEnvironment,
        alternate: MavrykEnvironment,
    ) -> String {
        let mut selected = selected.supported_values();
        selected.sort_unstable();
        let mut alternate_networks = alternate.supported_values();
        alternate_networks.sort_unstable();
        format!(
            "\n\n\n\n////////////////////////////////////////// \
            \n//      !!! DEPRECATED {} !!!      //\
            \n////////////////////////////////////////// \
            \n// {:?} is automatically switched to {:?} \
            \n// Better use directly network(s): {:?} \
            \n// Possible problems: \
            \n// - deprecated network will be removed soon \
            \n//////////////////////////////////////////\n\n\n\n",
            title, selected, alternate_networks, alternate_networks
        )
    }
}

#[derive(Debug, Clone)]
pub struct ParseMavrykEnvironmentError(String);

impl FromStr for MavrykEnvironment {
    type Err = ParseMavrykEnvironmentError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_ascii_lowercase();
        for sp in MavrykEnvironment::iter() {
            if sp.supported_values().contains(&s.as_str()) {
                return Ok(sp);
            }
        }

        Err(ParseMavrykEnvironmentError(format!(
            "Invalid variant name: {}",
            s
        )))
    }
}

/// Initializes hard-code default various configurations according to different Mavryk git branches (genesis_chain.ml, node_config_file.ml)
pub fn default_networks() -> HashMap<MavrykEnvironment, MavrykEnvironmentConfiguration> {
    let mut env: HashMap<MavrykEnvironment, MavrykEnvironmentConfiguration> = HashMap::new();

    env.insert(
        MavrykEnvironment::Mainnet,
        MavrykEnvironmentConfiguration {
            genesis: GenesisChain {
                time: "2018-06-30T16:07:32Z".to_string(),
                block: "BLockGenesisGenesisGenesisGenesisGenesisf79b5d1CoW2".to_string(),
                protocol: "Ps9mPmXaRzmzk35gbAYNCAw6UXdE2qoABTHbN2oEEc1qM7CwT9P".to_string(),
            },
            bootstrap_lookup_addresses: vec![
                "boot.tzbeta.net".to_string(),
                "116.202.172.21".to_string(), /* giganode_1 */
                "95.216.45.62".to_string(),   /* giganode_2 */
            ],
            version: "MAVRYK_MAINNET".to_string(),
            protocol_overrides: ProtocolOverrides {
                user_activated_upgrades: vec![
                    (
                        28082_i32,
                        "PsYLVpVvgbLhAhoqAkMFUo6gudkJ9weNXhUYCiLDzcUpFpkk8Wt".to_string(),
                    ),
                    (
                        204761_i32,
                        "PsddFKi32cMJ2qPjf43Qv5GDWLDPZb3T3bF6fLKiF5HtvHNU7aP".to_string(),
                    ),
                ],
                user_activated_protocol_overrides: vec![
                    (
                        "PsBABY5HQTSkA4297zNHfsZNKtxULfL18y95qb3m53QJiXGmrbU".to_string(),
                        "PsBabyM1eUXZseaJdmXFApDSBqj8YBfwELoxZHHW77EMcAbbwAS".to_string(),
                    ),
                    (
                        "PtEdoTezd3RHSC31mpxxo1npxFjoWWcFgQtxapi51Z8TLu6v6Uq".to_string(),
                        "PtEdo2ZkT9oKpimTah6x2embF25oss54njMuPzkJTEi5RqfdZFA".to_string(),
                    ),
                    (
                        "PtHangzHogokSuiMHemCuowEavgYTP8J5qQ9fQS793MHYFpCY3r".to_string(),
                        "PtHangz2aRngywmSRGGvrcTyMbbdpWdpFKuS4uMWxg2RaH9i1qx".to_string(),
                    ),
                ],
            },
            enable_testchain: false,
            patch_context_genesis_parameters: None,
        },
    );

    env.insert(
        MavrykEnvironment::Zeronet,
        MavrykEnvironmentConfiguration {
            genesis: GenesisChain {
                time: "2019-08-06T15:18:56Z".to_string(),
                block: "BLockGenesisGenesisGenesisGenesisGenesiscde8db4cX94".to_string(),
                protocol: "PtBMwNZT94N7gXKw4i273CKcSaBrrBnqnt3RATExNKr9KNX2USV".to_string(),
            },
            bootstrap_lookup_addresses: vec![
                "bootstrap.zeronet.fun".to_string(),
                "bootzero.tzbeta.net".to_string(),
            ],
            version: "MAVRYK_ZERONET_2019-08-06T15:18:56Z".to_string(),
            protocol_overrides: ProtocolOverrides {
                user_activated_upgrades: vec![],
                user_activated_protocol_overrides: vec![],
            },
            enable_testchain: true,
            patch_context_genesis_parameters: None,
        },
    );

    env.insert(
        MavrykEnvironment::Alphanet,
        MavrykEnvironmentConfiguration {
            genesis: GenesisChain {
                time: "2018-11-30T15:30:56Z".to_string(),
                block: "BLockGenesisGenesisGenesisGenesisGenesisb83baZgbyZe".to_string(),
                protocol: "Ps6mwMrF2ER2s51cp9yYpjDcuzQjsc2yAz8bQsRgdaRxw4Fk95H".to_string(),
            },
            bootstrap_lookup_addresses: vec![
                "boot.tzalpha.net".to_string(),
                "bootalpha.tzbeta.net".to_string(),
            ],
            version: "MAVRYK_ALPHANET_2018-11-30T15:30:56Z".to_string(),
            protocol_overrides: ProtocolOverrides {
                user_activated_upgrades: vec![],
                user_activated_protocol_overrides: vec![],
            },
            enable_testchain: false,
            patch_context_genesis_parameters: None,
        },
    );

    env.insert(
        MavrykEnvironment::Babylonnet,
        MavrykEnvironmentConfiguration {
            genesis: GenesisChain {
                time: "2019-09-27T07:43:32Z".to_string(),
                block: "BLockGenesisGenesisGenesisGenesisGenesisd1f7bcGMoXy".to_string(),
                protocol: "PtBMwNZT94N7gXKw4i273CKcSaBrrBnqnt3RATExNKr9KNX2USV".to_string(),
            },
            bootstrap_lookup_addresses: vec![
                "35.246.251.120".to_string(),
                "34.89.154.253".to_string(),
                "babylonnet.kaml.fr".to_string(),
            ],
            version: "MAVRYK_ALPHANET_BABYLON_2019-09-27T07:43:32Z".to_string(),
            protocol_overrides: ProtocolOverrides {
                user_activated_upgrades: vec![],
                user_activated_protocol_overrides: vec![],
            },
            enable_testchain: true,
            patch_context_genesis_parameters: None,
        },
    );

    env.insert(
        MavrykEnvironment::Carthagenet,
        MavrykEnvironmentConfiguration {
            genesis: GenesisChain {
                time: "2019-11-28T13:02:13Z".to_string(),
                block: "BLockGenesisGenesisGenesisGenesisGenesisd6f5afWyME7".to_string(),
                protocol: "PtYuensgYBb3G3x1hLLbCmcav8ue8Kyd2khADcL5LsT5R1hcXex".to_string(),
            },
            bootstrap_lookup_addresses: vec![
                "34.76.169.218".to_string(),
                "34.90.24.160".to_string(),
                "carthagenet.kaml.fr".to_string(),
                "104.248.136.94".to_string(),
            ],
            version: "MAVRYK_ALPHANET_CARTHAGE_2019-11-28T13:02:13Z".to_string(),
            protocol_overrides: ProtocolOverrides {
                user_activated_upgrades: vec![],
                user_activated_protocol_overrides: vec![],
            },
            enable_testchain: true,
            patch_context_genesis_parameters: None,
        },
    );

    env.insert(MavrykEnvironment::Delphinet, MavrykEnvironmentConfiguration {
        genesis: GenesisChain {
            time: "2020-09-04T07:08:53Z".to_string(),
            block: "BLockGenesisGenesisGenesisGenesisGenesis355e8bjkYPv".to_string(),
            protocol: "PtYuensgYBb3G3x1hLLbCmcav8ue8Kyd2khADcL5LsT5R1hcXex".to_string(),
        },
        bootstrap_lookup_addresses: vec![
            "delphinet.mavryk.co.il".to_string(),
            "delphinet.smartpy.io".to_string(),
            "delphinet.kaml.fr".to_string(),
            "13.53.41.201".to_string(),
        ],
        version: "MAVRYK_DELPHINET_2020-09-04T07:08:53Z".to_string(),
        protocol_overrides: ProtocolOverrides {
            user_activated_upgrades: vec![],
            user_activated_protocol_overrides: vec![],
        },
        enable_testchain: true,
        patch_context_genesis_parameters: Some(PatchContext {
            key: "sandbox_parameter".to_string(),
            json: r#"{ "genesis_pubkey": "edpkugeDwmwuwyyD3Q5enapgEYDxZLtEUFFSrvVwXASQMVEqsvTqWu" }"#.to_string(),
        }),
    });

    // TODO: remove after florence support
    // env.insert(MavrykEnvironment::Edonet, MavrykEnvironmentConfiguration {
    //     genesis: GenesisChain {
    //         time: "2020-11-30T12:00:00Z".to_string(),
    //         block: "BLockGenesisGenesisGenesisGenesisGenesis2431bbUwV2a".to_string(),
    //         protocol: "PtYuensgYBb3G3x1hLLbCmcav8ue8Kyd2khADcL5LsT5R1hcXex".to_string(),
    //     },
    //     bootstrap_lookup_addresses: vec![
    //         "51.75.246.56:9733".to_string(),
    //         "edonet.mavryk.co.il".to_string(),
    //         "46.245.179.161:9733".to_string(),
    //         "edonet.smartpy.io".to_string(),
    //         "188.40.128.216:29732".to_string(),
    //         "51.79.165.131".to_string(),
    //         "edonet.boot.mavryktaquito.io".to_string(),
    //         "95.216.228.228:9733".to_string(),
    //     ],
    //     version: "MAVRYK_EDONET_2020-11-30T12:00:00Z".to_string(),
    //     protocol_overrides: ProtocolOverrides {
    //         user_activated_upgrades: vec![],
    //         user_activated_protocol_overrides: vec![],
    //     },
    //     enable_testchain: true,
    //     patch_context_genesis_parameters: Some(PatchContext {
    //         key: "sandbox_parameter".to_string(),
    //         json: r#"{ "genesis_pubkey": "edpkugeDwmwuwyyD3Q5enapgEYDxZLtEUFFSrvVwXASQMVEqsvTqWu" }"#.to_string(),
    //     }),
    // });

    let edo2net_cfg = MavrykEnvironmentConfiguration {
        genesis: GenesisChain {
            time: "2021-02-11T14:00:00Z".to_string(),
            block: "BLockGenesisGenesisGenesisGenesisGenesisdae8bZxCCxh".to_string(),
            protocol: "PtYuensgYBb3G3x1hLLbCmcav8ue8Kyd2khADcL5LsT5R1hcXex".to_string(),
        },
        bootstrap_lookup_addresses: vec![
            "edonet.mavryk.co.il".to_string(),
            "188.40.128.216:29732".to_string(),
            "51.79.165.131".to_string(),
            "edo2net.kaml.fr".to_string(),
            "edonet2.smartpy.io".to_string(),
            "edonetb.boot.mavryktaquito.io".to_string(),
        ],
        version: "MAVRYK_EDO2NET_2021-02-11T14:00:00Z".to_string(),
        protocol_overrides: ProtocolOverrides {
            user_activated_upgrades: vec![],
            user_activated_protocol_overrides: vec![],
        },
        enable_testchain: true,
        patch_context_genesis_parameters: Some(PatchContext {
            key: "sandbox_parameter".to_string(),
            json:
                r#"{ "genesis_pubkey": "edpkugeDwmwuwyyD3Q5enapgEYDxZLtEUFFSrvVwXASQMVEqsvTqWu" }"#
                    .to_string(),
        }),
    };

    // TODO: for edo/edonet we redirect to edo2net, because edo/edonet is deprecated and not working
    // TODO: remove after florence support
    env.insert(MavrykEnvironment::Edonet, edo2net_cfg.clone());
    env.insert(MavrykEnvironment::Edo2net, edo2net_cfg);

    env.insert(MavrykEnvironment::Florencenet, MavrykEnvironmentConfiguration {
        genesis: GenesisChain {
            time: "2021-03-04T20:00:00Z".to_string(),
            block: "BMFCHw1mv3A71KpTuGD3MoFnkHk9wvTYjUzuR9QqiUumKGFG6pM".to_string(),
            protocol: "PtYuensgYBb3G3x1hLLbCmcav8ue8Kyd2khADcL5LsT5R1hcXex".to_string(),
        },
        bootstrap_lookup_addresses: vec![
            "florencenoba.tznode.net".to_string(),
            "florencenobanet.kaml.fr".to_string(),
            "florencenobanet.mavryk.co.il".to_string(),
            "florencenobanet.boot.tez.ie".to_string(),
            "florencenobanet.smartpy.io:9733".to_string(),
        ],
        version: "MAVRYK_FLORENCENOBANET_2021-03-04T20:00:00Z".to_string(),
        protocol_overrides: ProtocolOverrides {
            user_activated_upgrades: vec![],
            user_activated_protocol_overrides: vec![],
        },
        enable_testchain: true,
        patch_context_genesis_parameters: Some(PatchContext {
            key: "sandbox_parameter".to_string(),
            json: r#"{ "genesis_pubkey": "edpkuix6Lv8vnrz6uDe1w8uaXY7YktitAxn6EHdy2jdzq5n5hZo94n" }"#.to_string(),
        }),
    });

    env.insert(MavrykEnvironment::Granadanet, MavrykEnvironmentConfiguration {
        genesis: GenesisChain {
            time: "2021-05-21T15:00:00Z".to_string(),
            block: "BLockGenesisGenesisGenesisGenesisGenesisd4299hBGVoU".to_string(),
            protocol: "PtYuensgYBb3G3x1hLLbCmcav8ue8Kyd2khADcL5LsT5R1hcXex".to_string(),
        },
        bootstrap_lookup_addresses: vec![
            "granadanet.smartpy.io".to_string(),
            "granadanet.mavryk.co.il".to_string(),
            "granadanet.kaml.fr".to_string(),
        ],
        version: "MAVRYK_GRANADANET_2021-05-21T15:00:00Z".to_string(),
        protocol_overrides: ProtocolOverrides {
            user_activated_upgrades: vec![
                (
                    4095_i32,
                    "PtGRANADsDU8R9daYKAgWnQYAJ64omN1o3KMGVCykShA97vQbvV".to_string(),
                ),
            ],
            user_activated_protocol_overrides: vec![],
        },
        enable_testchain: true,
        patch_context_genesis_parameters: Some(PatchContext {
            key: "sandbox_parameter".to_string(),
            json: r#"{ "genesis_pubkey": "edpkuix6Lv8vnrz6uDe1w8uaXY7YktitAxn6EHdy2jdzq5n5hZo94n" }"#.to_string(),
        }),
    });

    env.insert(MavrykEnvironment::Hangzhounet, MavrykEnvironmentConfiguration {
        genesis: GenesisChain {
            time: "2021-11-04T15:00:00Z".to_string(),
            block: "BLockGenesisGenesisGenesisGenesisGenesis7e8c4d4snJW".to_string(),
            protocol: "Ps9mPmXaRzmzk35gbAYNCAw6UXdE2qoABTHbN2oEEc1qM7CwT9P".to_string(),
        },
        bootstrap_lookup_addresses: vec![
            "hangzhounet.teztnets.xyz".to_string(),
            "hangzhounet.kaml.fr".to_string(),
            "hangzhounet.smartpy.io".to_string(),
            "hangzhounet.mavryk.co.il".to_string(),
            "hangzhounet.boot.tez.ie".to_string(),
        ],
        version: "MAVRYK_HANGZHOUNET_2021-11-04T15:00:00Z".to_string(),
        protocol_overrides: ProtocolOverrides {
            user_activated_upgrades: vec![
                (
                    8191_i32,
                    "PtHangz2aRngywmSRGGvrcTyMbbdpWdpFKuS4uMWxg2RaH9i1qx".to_string(),
                ),
            ],
            user_activated_protocol_overrides: vec![],
        },
        enable_testchain: true,
        patch_context_genesis_parameters: Some(PatchContext {
            key: "sandbox_parameter".to_string(),
            json: r#"{ "genesis_pubkey": "edpkuYLienS3Xdt5c1vfRX1ibMxQuvfM67ByhJ9nmRYYKGAAoTq1UC" }"#.to_string(),
        }),
    });

    env.insert(MavrykEnvironment::Ithacanet, MavrykEnvironmentConfiguration {
        genesis: GenesisChain {
            time: "2022-01-25T15:00:00Z".to_string(),
            block: "BLockGenesisGenesisGenesisGenesisGenesis1db77eJNeJ9".to_string(),
            protocol: "Ps9mPmXaRzmzk35gbAYNCAw6UXdE2qoABTHbN2oEEc1qM7CwT9P".to_string(),
        },
        bootstrap_lookup_addresses: vec![
            "ithacanet.teztnets.xyz".to_string(),
            "ithacanet.smartpy.io".to_string(),
            "ithacanet.kaml.fr".to_string(),
            "ithacanet.boot.ecadinfra.com".to_string(),
        ],
        version: "MAVRYK_ITHACANET_2022-01-25T15:00:00Z".to_string(),
        protocol_overrides: ProtocolOverrides {
            user_activated_upgrades: vec![
                (
                    8191_i32,
                    "Psithaca2MLRFYargivpo7YvUr7wUDqyxrdhC5CQq78mRvimz6A".to_string(),
                ),
            ],
            user_activated_protocol_overrides: vec![],
        },
        enable_testchain: true,
        patch_context_genesis_parameters: Some(PatchContext {
            key: "sandbox_parameter".to_string(),
            json: r#"{ "genesis_pubkey": "edpkuYLienS3Xdt5c1vfRX1ibMxQuvfM67ByhJ9nmRYYKGAAoTq1UC" }"#.to_string(),
        }),
    });

    env.insert(MavrykEnvironment::Jakartanet, MavrykEnvironmentConfiguration {
        genesis: GenesisChain {
            time: "2022-04-27T15:00:00Z".to_string(),
            block: "BLockGenesisGenesisGenesisGenesisGenesisbd16dciJxo9".to_string(),
            protocol: "Ps9mPmXaRzmzk35gbAYNCAw6UXdE2qoABTHbN2oEEc1qM7CwT9P".to_string(),
        },
        bootstrap_lookup_addresses: vec![
            "jakartanet.teztnets.xyz".to_string(),
            "jakartanet.boot.ecadinfra.com".to_string(),
            "jakartanet.kaml.fr".to_string(),
            "jakartanet.visualtez.com".to_string(),
        ],
        version: "MAVRYK_JAKARTANET_2022-04-27T15:00:00Z".to_string(),
        protocol_overrides: ProtocolOverrides {
            user_activated_upgrades: vec![
                (
                    8192_i32,
                    "PtJakart2xVj7pYXJBXrqHgd82rdkLey5ZeeGwDgPp9rhQUbSqY".to_string(),
                ),
            ],
            user_activated_protocol_overrides: vec![],
        },
        enable_testchain: true,
        patch_context_genesis_parameters: Some(PatchContext {
            key: "sandbox_parameter".to_string(),
            json: r#"{ "genesis_pubkey": "edpkuYLienS3Xdt5c1vfRX1ibMxQuvfM67ByhJ9nmRYYKGAAoTq1UC" }"#.to_string(),
        }),
    });

    env.insert(MavrykEnvironment::Sandbox, MavrykEnvironmentConfiguration {
        genesis: GenesisChain {
            time: "2018-06-30T16:07:32Z".to_string(),
            block: "BLockGenesisGenesisGenesisGenesisGenesisf79b5d1CoW2".to_string(),
            protocol: "ProtoGenesisGenesisGenesisGenesisGenesisGenesk612im".to_string(),
        },
        bootstrap_lookup_addresses: vec![],
        version: "SANDBOXED_MAVRYK".to_string(),
        protocol_overrides: ProtocolOverrides {
            user_activated_upgrades: vec![],
            user_activated_protocol_overrides: vec![],
        },
        enable_testchain: false,
        patch_context_genesis_parameters: Some(PatchContext {
            key: "sandbox_parameter".to_string(),
            json: r#"{ "genesis_pubkey": "edpkuSLWfVU1Vq7Jg9FucPyKmma6otcMHac9zG4oU1KMHSTBpJuGQ2" }"#.to_string(),
        }),
    });

    env
}

/// Possible errors for environment
#[derive(Debug, Error)]
pub enum MavrykEnvironmentError {
    #[error("Invalid block hash: {hash}, reason: {error:?}")]
    InvalidBlockHash {
        hash: String,
        error: FromBase58CheckError,
    },
    #[error("Invalid protocol hash: {hash}, reason: {error:?}")]
    InvalidProtocolHash {
        hash: String,
        error: FromBase58CheckError,
    },
    #[error("Invalid time: {time}, reason: {error:?}")]
    InvalidTime {
        time: String,
        error: time::error::Parse,
    },
    #[error("Blake2b digest error")]
    Blake2bError,
}

impl From<Blake2bError> for MavrykEnvironmentError {
    fn from(_: Blake2bError) -> Self {
        MavrykEnvironmentError::Blake2bError
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GenesisAdditionalData {
    pub max_operations_ttl: u16,
    pub last_allowed_fork_level: i32,
    pub protocol_hash: ProtocolHash,
    pub next_protocol_hash: ProtocolHash,
}

/// Structure holding all environment specific crucial information - according to different Mavryk Gitlab branches
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct MavrykEnvironmentConfiguration {
    /// Genesis information - see genesis_chain.ml
    pub genesis: GenesisChain,
    /// Adresses used for initial bootstrap, if no peers are configured - see node_config_file.ml
    pub bootstrap_lookup_addresses: Vec<String>,
    /// chain_name - see distributed_db_version.ml
    pub version: String,
    /// protocol overrides - see block_header.ml
    pub protocol_overrides: ProtocolOverrides,
    /// if network has enabled switching test chains by default
    pub enable_testchain: bool,
    /// some networks could require patching context for genesis - like to change genesis key...
    /// (also this can be overriden on startup with cmd args)
    pub patch_context_genesis_parameters: Option<PatchContext>,
}

#[derive(Error, Debug)]
pub enum MavrykNetworkConfigurationError {
    #[error("I/O error: {reason}")]
    IoError { reason: io::Error },

    #[error("JSON config parsing error: {reason}")]
    ParseError { reason: serde_json::Error },
}

impl From<io::Error> for MavrykNetworkConfigurationError {
    fn from(reason: io::Error) -> Self {
        Self::IoError { reason }
    }
}

impl From<serde_json::Error> for MavrykNetworkConfigurationError {
    fn from(reason: serde_json::Error) -> Self {
        Self::ParseError { reason }
    }
}

impl MavrykEnvironmentConfiguration {
    /// Loads a custom network configuration from an mavkit-formatted configuration file
    pub fn try_from_config_file<P: AsRef<Path>>(
        path: P,
    ) -> Result<Self, MavrykNetworkConfigurationError> {
        let contents = fs::read_to_string(path)?;
        Self::try_from_json(&contents)
    }

    pub fn try_from_json(json: &str) -> Result<Self, MavrykNetworkConfigurationError> {
        let mavkit_config: MavkitConfig = serde_json::from_str(json)?;
        mavkit_config.take_network()
    }

    /// Resolves genesis hash from configuration of GenesisChain.block
    pub fn genesis_header_hash(&self) -> Result<BlockHash, MavrykEnvironmentError> {
        BlockHash::from_base58_check(&self.genesis.block).map_err(|e| {
            MavrykEnvironmentError::InvalidBlockHash {
                hash: self.genesis.block.clone(),
                error: e,
            }
        })
    }

    /// Resolves main chain_id, which is computed from genesis header
    pub fn main_chain_id(&self) -> Result<ChainId, MavrykEnvironmentError> {
        chain_id_from_block_hash(&self.genesis_header_hash()?).map_err(|e| e.into())
    }

    /// Resolves genesis protocol
    pub fn genesis_protocol(&self) -> Result<ProtocolHash, MavrykEnvironmentError> {
        self.genesis.protocol.as_str().try_into().map_err(|e| {
            MavrykEnvironmentError::InvalidProtocolHash {
                hash: self.genesis.protocol.clone(),
                error: e,
            }
        })
    }

    pub fn genesis_time(&self) -> Result<i64, MavrykEnvironmentError> {
        parse_from_rfc3339(&self.genesis.time)
    }

    /// Returns initialized default genesis header
    pub fn genesis_header(
        &self,
        context_hash: ContextHash,
        operation_list_list_hash: OperationListListHash,
    ) -> Result<BlockHeader, MavrykEnvironmentError> {
        // genesis predecessor is genesis
        let genesis_hash = self.genesis_header_hash()?;
        let genesis_time: i64 = self.genesis_time()?;

        Ok(BlockHeaderBuilder::default()
            .level(0)
            .proto(0)
            .predecessor(genesis_hash)
            .timestamp(genesis_time.into())
            .validation_pass(0)
            .operations_hash(operation_list_list_hash)
            .fitness(vec![].into())
            .context(context_hash)
            .protocol_data(vec![].into())
            .build()
            .unwrap())
    }

    pub fn genesis_additional_data(&self) -> Result<GenesisAdditionalData, MavrykEnvironmentError> {
        Ok(GenesisAdditionalData {
            max_operations_ttl: 0,
            last_allowed_fork_level: 0,
            protocol_hash: ProtocolHash::from_base58_check(PROTOCOL_HASH_ZERO_BASE58_CHECK)
                .map_err(|e| MavrykEnvironmentError::InvalidProtocolHash {
                    hash: self.genesis.protocol.clone(),
                    error: e,
                })?,
            next_protocol_hash: self.genesis_protocol()?,
        })
    }
}

fn parse_from_rfc3339(time: &str) -> Result<i64, MavrykEnvironmentError> {
    OffsetDateTime::parse(time, &Rfc3339)
        .map(OffsetDateTime::unix_timestamp)
        .map_err(|error| MavrykEnvironmentError::InvalidTime {
            time: String::from(time),
            error,
        })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddrParseError(String);

/// Parse ip:port from str, if port is not present, then uses default
///
/// IPv6 addresses must be enclosed with brackets '[...]'
///
/// Note: ip could be <IP> or domain/host name
/// Note: see p2p_point.ml -> parse_addr_port
pub fn parse_bootstrap_addr_port(
    addr: &str,
    default_port: u16,
) -> Result<(String, u16), AddrParseError> {
    let (addr, port) = if addr.starts_with('[') {
        // handle ipv6
        match addr.rfind(']') {
            Some(idx) => {
                // parse addr
                let addr_part = &addr[1..idx];
                let port_part = if idx >= addr.len() - 1 {
                    // no port
                    None
                } else if addr.get((idx + 1)..(idx + 2)) != Some(":") {
                    return Err(AddrParseError(format!(
                        "Invalid value '{}' - invalid part with '<:PORT>'",
                        addr
                    )));
                } else {
                    // try get port part after ':'
                    match addr.get((idx + 2)..) {
                        Some(port_part) => Some(port_part),
                        None => {
                            return Err(AddrParseError(format!(
                                "Invalid value '{}' - invalid port part",
                                addr
                            )))
                        }
                    }
                };
                (addr_part, port_part)
            }
            None => {
                return Err(AddrParseError(format!(
                    "Invalid value '{}' - missing closing ']'",
                    addr
                )))
            }
        }
    } else {
        if addr.rfind(']').is_some() {
            return Err(AddrParseError(format!(
                "Invalid value '{}' - missing starting '['",
                addr
            )));
        }
        match addr.match_indices(':').count() {
            0 => (addr, None),
            1 => {
                let mut split = addr.split(':');
                let addr_part = match split.next() {
                    Some(s) => s,
                    None => {
                        return Err(AddrParseError(format!(
                            "Invalid value '{}' - invalid addr part",
                            addr
                        )))
                    }
                };
                let port_part = match split.next() {
                    Some(s) => s,
                    None => {
                        return Err(AddrParseError(format!(
                            "Invalid value '{}' - invalid port part",
                            addr
                        )))
                    }
                };
                (addr_part, Some(port_part))
            }
            _ => {
                return Err(AddrParseError(format!(
                    "Invalid value '{}' - Ipv6 addr should be wrapped with '[...]'",
                    addr
                )))
            }
        }
    };

    // check port
    let port = match port {
        Some(p) => match p.parse::<u16>() {
            Ok(p) => p,
            Err(e) => {
                return Err(AddrParseError(format!(
                    "Invalid value '{}' - invalid port value '{}', reason: {}",
                    addr, p, e
                )))
            }
        },
        None => default_port,
    };

    Ok((addr.to_string(), port))
}

#[derive(Debug, Clone)]
pub struct ZcashParams {
    pub init_sapling_spend_params_file: PathBuf,
    pub init_sapling_output_params_file: PathBuf,
}

impl ZcashParams {
    pub const SAPLING_SPEND_PARAMS_FILE_NAME: &'static str = "sapling-spend.params";
    pub const SAPLING_OUTPUT_PARAMS_FILE_NAME: &'static str = "sapling-output.params";

    pub fn description(&self, args: &str) -> String {
        format!(
            "\nOne of candidate dirs {:?} should contains files: [{}, {}],\n \
            1. these files could be created on startup, but you must provide correct existing paths as arguments: {},\n \
            2. or you may download https://raw.githubusercontent.com/zcash/zcash/master/zcutil/fetch-params.sh and set it up",
            self.candidate_dirs(),
            ZcashParams::SAPLING_SPEND_PARAMS_FILE_NAME,
            ZcashParams::SAPLING_OUTPUT_PARAMS_FILE_NAME,
            args,
        )
    }

    fn candidate_dirs(&self) -> Vec<PathBuf> {
        let mut candidates = Vec::new();

        // first check opam switch (this is default path of Mavryk installation)
        if let Some(opam_switch_path) = env::var_os("OPAM_SWITCH_PREFIX") {
            candidates.push(PathBuf::from(opam_switch_path).join("share/zcash-params"));
        }

        // home dir or root
        let home_dir = env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/"));
        candidates.push(home_dir.join(".zcash-params"));

        // data dirs
        let mut data_dirs = vec![match env::var_os("XDG_DATA_HOME") {
            Some(xdg_data_home) => PathBuf::from(xdg_data_home),
            None => home_dir.join(".local/share/"),
        }];
        data_dirs.extend(
            env::var_os("XDG_DATA_DIRS")
                .unwrap_or_else(|| OsString::from("/usr/local/share/:/usr/share/"))
                .as_os_str()
                .to_str()
                .unwrap_or("/usr/local/share/:/usr/share/")
                .split(':')
                .map(PathBuf::from),
        );
        candidates.extend(data_dirs.into_iter().map(|dir| dir.join("zcash-params")));

        candidates
    }

    /// Checks correctly setup environment OS for zcash-params sapling.
    /// Note: According to Mavryk ocaml rustzcash.ml
    pub fn assert_zcash_params(&self, log: &Logger) -> Result<(), anyhow::Error> {
        // select candidate dirs
        let candidates = self.candidate_dirs();

        // check if anybody contains required files
        debug!(log, "Possible candidates for zcash-params found"; "candidates" => format!("{:?}", candidates));

        let zcash_params_dir_found = {
            let mut found = false;
            for candidate in candidates {
                let spend_path = candidate.join(Self::SAPLING_SPEND_PARAMS_FILE_NAME);
                let output_path = candidate.join(Self::SAPLING_OUTPUT_PARAMS_FILE_NAME);
                if spend_path.exists() && output_path.exists() {
                    info!(log, "Found existing zcash-params files";
                               "candidate_dir" => format!("{:?}", candidate),
                               "spend_path" => format!("{:?}", spend_path),
                               "output_path" => format!("{:?}", output_path));
                    found = true;
                }
            }
            found
        };

        // if not found, if we need to create it from configured init files (because protocol expected it)
        if !zcash_params_dir_found {
            // check init files
            if !self.init_sapling_spend_params_file.exists() {
                return Err(anyhow::format_err!(
                    "File not found for init_sapling_spend_params_file: {:?}",
                    self.init_sapling_spend_params_file
                ));
            }
            if !self.init_sapling_output_params_file.exists() {
                return Err(anyhow::format_err!(
                    "File not found for init_sapling_output_params_file: {:?}",
                    self.init_sapling_output_params_file
                ));
            }

            // we initialize zcash-params in user's home dir (it is one of the candidates)
            let home_dir = env::var_os("HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("/"));
            let zcash_param_dir = home_dir.join(".zcash-params");
            if !zcash_param_dir.exists() {
                info!(log, "Creating new zcash-params dir"; "dir" => format!("{:?}", zcash_param_dir));
                fs::create_dir_all(&zcash_param_dir)?;
            }

            info!(log, "Using configured init files for zcash-params";
                           "spend_path" => format!("{:?}", self.init_sapling_spend_params_file),
                           "output_path" => format!("{:?}", self.init_sapling_output_params_file));

            // copy init files to dest
            let dest_spend_path = zcash_param_dir.join(Self::SAPLING_SPEND_PARAMS_FILE_NAME);
            let dest_output_path = zcash_param_dir.join(Self::SAPLING_OUTPUT_PARAMS_FILE_NAME);
            fs::copy(&self.init_sapling_spend_params_file, &dest_spend_path)?;
            fs::copy(&self.init_sapling_output_params_file, &dest_output_path)?;
            info!(log, "Sapling zcash-params files were created";
                           "dir" => format!("{:?}", zcash_param_dir),
                           "spend_path" => format!("{:?}", dest_spend_path),
                           "output_path" => format!("{:?}", dest_output_path));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use mavryk_messages::{p2p::encoding::limits::CHAIN_NAME_MAX_LENGTH, ts_to_rfc3339};

    use super::*;

    #[test]
    fn encoded_decoded_timestamp() -> Result<(), anyhow::Error> {
        let dt = parse_from_rfc3339("2019-11-28T13:02:13Z")?;
        let decoded = ts_to_rfc3339(dt).unwrap();
        let expected = "2019-11-28T13:02:13Z".to_string();

        assert_eq!(expected, decoded);
        Ok(())
    }

    #[test]
    fn test_parse_bootstrap_addr_port() -> Result<(), AddrParseError> {
        let (addr, port) = parse_bootstrap_addr_port("edonet.mavryk.co.il", 5)?;
        assert_eq!("edonet.mavryk.co.il", addr);
        assert_eq!(5, port);

        let (addr, port) = parse_bootstrap_addr_port("188.40.128.216:29732", 5)?;
        assert_eq!("188.40.128.216", addr);
        assert_eq!(29732, port);

        let (addr, port) = parse_bootstrap_addr_port("[fe80:e828:209d:20e:c0ae::]:375", 5)?;
        assert_eq!("fe80:e828:209d:20e:c0ae::", addr);
        assert_eq!(375, port);

        let (addr, port) = parse_bootstrap_addr_port("[2a01:4f8:171:1f2d::2]:9732", 5)?;
        assert_eq!("2a01:4f8:171:1f2d::2", addr);
        assert_eq!(9732, port);

        assert!(parse_bootstrap_addr_port("[fe80:e828:209d:20e:c0ae::", 5).is_err());
        assert!(parse_bootstrap_addr_port("fe80:e828:209d:20e:c0ae::]:375", 5).is_err());
        assert!(parse_bootstrap_addr_port("fe80:e828:209d:20e:c0ae::]:", 5).is_err());
        assert!(parse_bootstrap_addr_port("fe80:e828:209d:20e:c0ae:375", 5).is_err());
        assert!(parse_bootstrap_addr_port("188.40.128.216:a", 5).is_err());
        assert!(parse_bootstrap_addr_port("188.40.128.216:", 5).is_err());

        Ok(())
    }

    #[test]
    fn test_parse_bootstrap_addr_port_for_all_environment() {
        let default_networks = default_networks();

        MavrykEnvironment::iter()
            .filter(|te| *te != MavrykEnvironment::Custom)
            .for_each(|net| {
                let mavryk_env: &MavrykEnvironmentConfiguration = default_networks
                    .get(&net)
                    .unwrap_or_else(|| panic!("no mavryk environment configured for: {:?}", &net));

                mavryk_env
                    .bootstrap_lookup_addresses
                    .iter()
                    .for_each(|addr| assert!(parse_bootstrap_addr_port(addr, 1111).is_ok()));
            });
    }

    #[test]
    fn test_network_version_length() {
        let default_networks = default_networks();

        MavrykEnvironment::iter()
            .filter(|te| *te != MavrykEnvironment::Custom)
            .for_each(|net| {
                let mavryk_env: &MavrykEnvironmentConfiguration = default_networks
                    .get(&net)
                    .unwrap_or_else(|| panic!("no mavryk environment configured for: {:?}", &net));

                assert!(
                    mavryk_env.version.len() <= CHAIN_NAME_MAX_LENGTH,
                    "The chain version {} does not fit into the CHAIN_NAME_MAX_LENGTH value {}",
                    mavryk_env.version.len(),
                    CHAIN_NAME_MAX_LENGTH
                );
            });
    }

    #[test]
    fn test_mavryk_environment_configuration_from_custom_network_json() {
        let json = r#"{
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

        let mavryk_env = MavrykEnvironmentConfiguration::try_from_json(json).unwrap();
        let expected = MavrykEnvironmentConfiguration {
            genesis: GenesisChain {
                time: "2018-06-30T16:07:32Z".into(),
                block: "BLockGenesisGenesisGenesisGenesisGenesisf79b5d1CoW2".into(),
                protocol: "PtYuensgYBb3G3x1hLLbCmcav8ue8Kyd2khADcL5LsT5R1hcXex".into(),
            },
            bootstrap_lookup_addresses: vec![],
            version: "SANDBOXED_MAVRYK".into(),
            protocol_overrides: ProtocolOverrides {
                user_activated_upgrades: vec![],
                user_activated_protocol_overrides: vec![],
            },
            enable_testchain: false,
            patch_context_genesis_parameters: Some(PatchContext {
                key: "sandbox_parameter".into(),
                json:
                    r#"{"genesis_pubkey":"edpkuJQjuxBndWiwNRFGndPaJATFVXsiDDyAfE4oHvUtu138w5LYRs"}"#
                        .into(),
            }),
        };

        assert_eq!(mavryk_env, expected);
    }
}
