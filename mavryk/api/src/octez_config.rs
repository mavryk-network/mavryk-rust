// Copyright (c) SimpleStaking, Viable Systems and Tezedge Contributors
// SPDX-License-Identifier: MIT

use std::collections::HashMap;

use serde::Deserialize;

use crate::environment::{MavrykEnvironmentConfiguration, MavrykNetworkConfigurationError};
use std::convert::{TryFrom, TryInto};
use mavryk_context_api::{GenesisChain, PatchContext, ProtocolOverrides};

#[derive(Deserialize, Debug)]
pub struct MavkitConfig {
    network: MavkitCustomNetwork,
}

impl MavkitConfig {
    pub fn take_network(
        self,
    ) -> Result<MavrykEnvironmentConfiguration, MavrykNetworkConfigurationError> {
        self.network.try_into()
    }
}

#[derive(Deserialize, Debug, Clone)]
struct MavkitCustomNetwork {
    pub chain_name: String,
    pub genesis: MavkitGenesisChain,
    #[allow(dead_code)]
    pub sandboxed_chain_name: String,
    #[serde(default)]
    pub default_bootstrap_peers: Vec<String>,
    pub genesis_parameters: Option<MavkitGenesisParameters>,
    #[serde(default)]
    pub user_activate_upgrades: Vec<UserActivatedProtocolUpgrades>,
    #[serde(default)]
    pub user_activate_protocol_overrides: Vec<UserActivatedProtocolOverride>,
}

#[derive(Deserialize, Debug, Clone)]
struct UserActivatedProtocolOverride {
    replaced_protocol: String,
    replacement_protocol: String,
}

#[derive(Deserialize, Debug, Clone)]
struct UserActivatedProtocolUpgrades {
    level: i32,
    replacement_protocol: String,
}

impl TryFrom<MavkitCustomNetwork> for MavrykEnvironmentConfiguration {
    type Error = MavrykNetworkConfigurationError;

    fn try_from(mavkit: MavkitCustomNetwork) -> Result<Self, Self::Error> {
        Ok(Self {
            genesis: mavkit.genesis.into(),
            bootstrap_lookup_addresses: mavkit.default_bootstrap_peers,
            version: mavkit.chain_name,
            protocol_overrides: ProtocolOverrides {
                user_activated_upgrades: mavkit
                    .user_activate_upgrades
                    .iter()
                    .map(|u| (u.level, u.replacement_protocol.clone()))
                    .collect(),
                user_activated_protocol_overrides: mavkit
                    .user_activate_protocol_overrides
                    .iter()
                    .map(|po| {
                        (
                            po.replaced_protocol.clone(),
                            po.replacement_protocol.clone(),
                        )
                    })
                    .collect(),
            },
            enable_testchain: false,
            patch_context_genesis_parameters: match mavkit.genesis_parameters {
                Some(gp) => Some(gp.try_into()?),
                None => None,
            },
        })
    }
}

#[derive(Deserialize, Debug, Clone)]
struct MavkitGenesisChain {
    pub timestamp: String,
    pub block: String,
    pub protocol: String,
}

impl From<MavkitGenesisChain> for GenesisChain {
    fn from(mavkit: MavkitGenesisChain) -> Self {
        Self {
            time: mavkit.timestamp,
            block: mavkit.block,
            protocol: mavkit.protocol,
        }
    }
}

fn mavkit_default_context_key() -> String {
    "sandbox_parameter".to_owned()
}

#[derive(Deserialize, Debug, Clone)]
struct MavkitGenesisParameters {
    #[serde(default = "mavkit_default_context_key")]
    context_key: String,
    #[serde(default)]
    values: HashMap<String, String>,
}

impl TryFrom<MavkitGenesisParameters> for PatchContext {
    type Error = MavrykNetworkConfigurationError;

    fn try_from(mavkit: MavkitGenesisParameters) -> Result<Self, Self::Error> {
        Ok(Self {
            key: mavkit.context_key,
            json: serde_json::to_string(&mavkit.values)
                .map_err(|e| MavrykNetworkConfigurationError::ParseError { reason: e })?,
        })
    }
}
