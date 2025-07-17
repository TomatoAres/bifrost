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

pub use asset_hub_rococo_runtime;

pub mod genesis;

// Substrate
use frame_support::traits::OnInitialize;

// Cumulus
use emulated_integration_tests_common::{
	impl_accounts_helpers_for_parachain, impl_assert_events_helpers_for_parachain,
	impl_assets_helpers_for_parachain, impl_assets_helpers_for_system_parachain,
	impl_bridge_helpers_for_chain, impl_foreign_assets_helpers_for_parachain,
	impl_xcm_helpers_for_parachain, impls::Parachain, xcm_emulator::decl_test_parachains,
};
use rococo_emulated_chain::Rococo;

// AssetHubRococo Parachain declaration
decl_test_parachains! {
	pub struct AssetHubRococo {
		genesis = genesis::genesis(),
		on_init = {
			asset_hub_rococo_runtime::AuraExt::on_initialize(1);
		},
		runtime = asset_hub_rococo_runtime,
		core = {
			XcmpMessageHandler: asset_hub_rococo_runtime::XcmpQueue,
			LocationToAccountId: asset_hub_rococo_runtime::xcm_config::LocationToAccountId,
			ParachainInfo: asset_hub_rococo_runtime::ParachainInfo,
			MessageOrigin: cumulus_primitives_core::AggregateMessageOrigin,
		},
		pallets = {
			System: asset_hub_rococo_runtime::System,
			PolkadotXcm: asset_hub_rococo_runtime::PolkadotXcm,
			Assets: asset_hub_rococo_runtime::Assets,
			ForeignAssets: asset_hub_rococo_runtime::ForeignAssets,
			PoolAssets: asset_hub_rococo_runtime::PoolAssets,
			AssetConversion: asset_hub_rococo_runtime::AssetConversion,
			Balances: asset_hub_rococo_runtime::Balances,
		}
	},
}

// AssetHubRococo implementation
impl_accounts_helpers_for_parachain!(AssetHubRococo);
impl_assert_events_helpers_for_parachain!(AssetHubRococo);
impl_assets_helpers_for_system_parachain!(AssetHubRococo, Rococo);
impl_assets_helpers_for_parachain!(AssetHubRococo);
impl_foreign_assets_helpers_for_parachain!(AssetHubRococo, xcm::v5::Location);
impl_xcm_helpers_for_parachain!(AssetHubRococo);
impl_bridge_helpers_for_chain!(
	AssetHubRococo,
	ParaPallet,
	PolkadotXcm,
	bp_bridge_hub_rococo::RuntimeCall::XcmOverBridgeHubWestend
);
