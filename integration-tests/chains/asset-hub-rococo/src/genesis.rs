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
use emulated_integration_tests_common::{
	accounts, build_genesis_storage, collators, PenpalASiblingSovereignAccount,
	PenpalATeleportableAssetLocation, PenpalBSiblingSovereignAccount,
	PenpalBTeleportableAssetLocation, RESERVABLE_ASSET_ID, SAFE_XCM_VERSION, USDT_ID,
};
use parachains_common::{AccountId, Balance};

pub const PARA_ID: u32 = 1000;
pub const ED: Balance = testnet_parachains_constants::rococo::currency::EXISTENTIAL_DEPOSIT;

parameter_types! {
	pub AssetHubRococoAssetOwner: AccountId = Keyring::Alice.to_account_id();
}

pub fn genesis() -> Storage {
	let genesis_config = asset_hub_rococo_runtime::RuntimeGenesisConfig {
		system: asset_hub_rococo_runtime::SystemConfig::default(),
		balances: asset_hub_rococo_runtime::BalancesConfig {
			balances: accounts::init_balances()
				.iter()
				.cloned()
				.map(|k| (k, ED * 4096 * 4096))
				.collect(),
		},
		parachain_info: asset_hub_rococo_runtime::ParachainInfoConfig {
			parachain_id: PARA_ID.into(),
			..Default::default()
		},
		collator_selection: asset_hub_rococo_runtime::CollatorSelectionConfig {
			invulnerables: collators::invulnerables()
				.iter()
				.cloned()
				.map(|(acc, _)| acc)
				.collect(),
			candidacy_bond: ED * 16,
			..Default::default()
		},
		session: asset_hub_rococo_runtime::SessionConfig {
			keys: collators::invulnerables()
				.into_iter()
				.map(|(acc, aura)| {
					(
						acc.clone(),                                    // account id
						acc,                                            // validator id
						asset_hub_rococo_runtime::SessionKeys { aura }, // session keys
					)
				})
				.collect(),
			..Default::default()
		},
		polkadot_xcm: asset_hub_rococo_runtime::PolkadotXcmConfig {
			safe_xcm_version: Some(SAFE_XCM_VERSION),
			..Default::default()
		},
		assets: asset_hub_rococo_runtime::AssetsConfig {
			assets: vec![
				(
					RESERVABLE_ASSET_ID,
					AssetHubRococoAssetOwner::get(),
					false,
					ED,
				),
				(USDT_ID, AssetHubRococoAssetOwner::get(), true, ED),
			],
			..Default::default()
		},
		foreign_assets: asset_hub_rococo_runtime::ForeignAssetsConfig {
			assets: vec![
				// PenpalA's teleportable asset representation
				(
					PenpalATeleportableAssetLocation::get(),
					PenpalASiblingSovereignAccount::get(),
					false,
					ED,
				),
				// PenpalB's teleportable asset representation
				(
					PenpalBTeleportableAssetLocation::get(),
					PenpalBSiblingSovereignAccount::get(),
					false,
					ED,
				),
			],
			..Default::default()
		},
		..Default::default()
	};

	build_genesis_storage(
		&genesis_config,
		asset_hub_rococo_runtime::WASM_BINARY.expect("WASM binary was not built, please build it!"),
	)
}
