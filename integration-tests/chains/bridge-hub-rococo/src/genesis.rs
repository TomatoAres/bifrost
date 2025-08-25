// This file is part of Bifrost.

// Copyright (C) Liebi Technologies PTE. LTD.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

// Substrate
use sp_core::storage::Storage;
use sp_keyring::Sr25519Keyring as Keyring;

// Cumulus
use emulated_integration_tests_common::{
	accounts, build_genesis_storage, collators, SAFE_XCM_VERSION,
};
use parachains_common::Balance;
use xcm::latest::{prelude::*, WESTEND_GENESIS_HASH};

pub const ASSETHUB_PARA_ID: u32 = 1000;
pub const PARA_ID: u32 = 1013;
pub const ED: Balance = testnet_parachains_constants::rococo::currency::EXISTENTIAL_DEPOSIT;

pub fn genesis() -> Storage {
	let genesis_config = bridge_hub_rococo_runtime::RuntimeGenesisConfig {
		system: bridge_hub_rococo_runtime::SystemConfig::default(),
		balances: bridge_hub_rococo_runtime::BalancesConfig {
			balances: accounts::init_balances()
				.iter()
				.cloned()
				.map(|k| (k, ED * 4096))
				.collect(),
			dev_accounts: None,
		},
		parachain_info: bridge_hub_rococo_runtime::ParachainInfoConfig {
			parachain_id: PARA_ID.into(),
			..Default::default()
		},
		collator_selection: bridge_hub_rococo_runtime::CollatorSelectionConfig {
			invulnerables: collators::invulnerables()
				.iter()
				.cloned()
				.map(|(acc, _)| acc)
				.collect(),
			candidacy_bond: ED * 16,
			..Default::default()
		},
		session: bridge_hub_rococo_runtime::SessionConfig {
			keys: collators::invulnerables()
				.into_iter()
				.map(|(acc, aura)| {
					(
						acc.clone(),                                     // account id
						acc,                                             // validator id
						bridge_hub_rococo_runtime::SessionKeys { aura }, // session keys
					)
				})
				.collect(),
			..Default::default()
		},
		polkadot_xcm: bridge_hub_rococo_runtime::PolkadotXcmConfig {
			safe_xcm_version: Some(SAFE_XCM_VERSION),
			..Default::default()
		},
		bridge_westend_grandpa: bridge_hub_rococo_runtime::BridgeWestendGrandpaConfig {
			owner: Some(Keyring::Bob.to_account_id()),
			..Default::default()
		},
		bridge_westend_messages: bridge_hub_rococo_runtime::BridgeWestendMessagesConfig {
			owner: Some(Keyring::Bob.to_account_id()),
			..Default::default()
		},
		xcm_over_bridge_hub_westend: bridge_hub_rococo_runtime::XcmOverBridgeHubWestendConfig {
			opened_bridges: vec![
				// open AHR -> AHW bridge
				(
					Location::new(1, [Parachain(1000)]),
					Junctions::from([ByGenesis(WESTEND_GENESIS_HASH).into(), Parachain(1000)]),
					Some(bp_messages::LegacyLaneId([0, 0, 0, 2])),
				),
			],
			..Default::default()
		},
		ethereum_system: bridge_hub_rococo_runtime::EthereumSystemConfig {
			para_id: PARA_ID.into(),
			asset_hub_para_id: ASSETHUB_PARA_ID.into(),
			..Default::default()
		},
		..Default::default()
	};

	build_genesis_storage(
		&genesis_config,
		bridge_hub_rococo_runtime::WASM_BINARY
			.expect("WASM binary was not built, please build it!"),
	)
}
