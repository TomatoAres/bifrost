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
use frame_support::parameter_types;
use frame_support::sp_runtime::FixedU128;
use sp_core::storage::Storage;
use sp_keyring::Sr25519Keyring as Keyring;

// Cumulus
use bifrost_primitives::currency::BNC;
use bifrost_primitives::{KSM, VKSM};
use bifrost_runtime_common::bridge_xcm_helper::DEFAULT_XCM_FEES_IK_PERSPECTIVE;
use emulated_integration_tests_common::{
	accounts, build_genesis_storage, collators, SAFE_XCM_VERSION,
};
use parachains_common::{AccountId, Balance};

pub const PARA_ID: u32 = 2001;
pub const ED: Balance = 10_000_000_000;

parameter_types! {
	pub AssetHubRococoAssetOwner: AccountId = Keyring::Alice.to_account_id();
}

pub fn genesis() -> Storage {
	let genesis_config = bifrost_kusama_runtime::RuntimeGenesisConfig {
		system: bifrost_kusama_runtime::SystemConfig::default(),
		balances: bifrost_kusama_runtime::BalancesConfig {
			balances: accounts::init_balances()
				.iter()
				.cloned()
				.map(|k| (k, ED * 4096 * 4096))
				.collect(),
			dev_accounts: None,
		},
		parachain_info: bifrost_kusama_runtime::ParachainInfoConfig {
			parachain_id: PARA_ID.into(),
			..Default::default()
		},
		session: bifrost_kusama_runtime::SessionConfig {
			keys: collators::invulnerables()
				.into_iter()
				.map(|(acc, aura)| {
					(
						acc.clone(),                                  // account id
						acc,                                          // validator id
						bifrost_kusama_runtime::SessionKeys { aura }, // session keys
					)
				})
				.collect(),
			..Default::default()
		},
		polkadot_xcm: bifrost_kusama_runtime::PolkadotXcmConfig {
			safe_xcm_version: Some(SAFE_XCM_VERSION),
			..Default::default()
		},
		asset_registry: bifrost_kusama_runtime::AssetRegistryConfig {
			currency: vec![
				(
					BNC,
					ED,
					Some((
						String::from("Bifrost Native Coin"),
						String::from("BNC"),
						12u8,
					)),
				),
				(
					KSM,
					1_000_000,
					Some((String::from("Kusama KSM"), String::from("KSM"), 12u8)),
				),
			],
			vcurrency: vec![VKSM],
			..Default::default()
		},
		prices: bifrost_kusama_runtime::PricesConfig {
			emergency_price: vec![
				(KSM, FixedU128::from_inner(15_000_000_000_000_000_000u128)),
				(BNC, FixedU128::from_inner(100_000_000_000_000_000u128)),
			],
			..Default::default()
		},
		pk_bridge: bifrost_kusama_runtime::PKBridgeConfig {
			bridge_config: bifrost_p_k_bridge::BridgeConfig {
				send_enabled: true,
				receive_enabled: true,
			},
			initial_xcm_fees: Some(DEFAULT_XCM_FEES_IK_PERSPECTIVE),
		},
		..Default::default()
	};
	build_genesis_storage(
		&genesis_config,
		bifrost_kusama_runtime::WASM_BINARY.expect("WASM binary was not built, please build it!"),
	)
}
