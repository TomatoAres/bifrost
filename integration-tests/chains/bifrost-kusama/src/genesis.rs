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
use sp_core::storage::Storage;
use sp_keyring::Sr25519Keyring as Keyring;

// Cumulus
use bifrost_primitives::currency::{BNC, DOT, DOT_U, GLMR};
use bifrost_primitives::CurrencyId::VToken2;
use bifrost_primitives::DOT_TOKEN_ID;
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
					DOT,
					1_000_000,
					Some((String::from("Polkadot DOT"), String::from("DOT"), 10u8)),
				),
				(
					GLMR,
					1_000_000_000_000,
					Some((
						String::from("Moonbeam Native Token"),
						String::from("GLMR"),
						18u8,
					)),
				),
				(
					DOT_U,
					1_000,
					Some((String::from("Tether USD"), String::from("USDT"), 6u8)),
				),
			],
			vcurrency: vec![VToken2(DOT_TOKEN_ID)],
			..Default::default()
		},
		..Default::default()
	};

	build_genesis_storage(
		&genesis_config,
		bifrost_kusama_runtime::WASM_BINARY.expect("WASM binary was not built, please build it!"),
	)
}
