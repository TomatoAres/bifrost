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

pub use asset_hub_westend_runtime;

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
use westend_emulated_chain::Westend;

// AssetHubWestend Parachain declaration
decl_test_parachains! {
	pub struct AssetHubWestend {
		genesis = genesis::genesis(),
		on_init = {
			asset_hub_westend_runtime::AuraExt::on_initialize(1);
		},
		runtime = asset_hub_westend_runtime,
		core = {
			XcmpMessageHandler: asset_hub_westend_runtime::XcmpQueue,
			LocationToAccountId: asset_hub_westend_runtime::xcm_config::LocationToAccountId,
			ParachainInfo: asset_hub_westend_runtime::ParachainInfo,
			MessageOrigin: cumulus_primitives_core::AggregateMessageOrigin,
		},
		pallets = {
			PolkadotXcm: asset_hub_westend_runtime::PolkadotXcm,
			Balances: asset_hub_westend_runtime::Balances,
			Assets: asset_hub_westend_runtime::Assets,
			ForeignAssets: asset_hub_westend_runtime::ForeignAssets,
			PoolAssets: asset_hub_westend_runtime::PoolAssets,
			AssetConversion: asset_hub_westend_runtime::AssetConversion,
		}
	},
}

// AssetHubWestend implementation
impl_accounts_helpers_for_parachain!(AssetHubWestend);
impl_assert_events_helpers_for_parachain!(AssetHubWestend);
impl_assets_helpers_for_system_parachain!(AssetHubWestend, Westend);
impl_assets_helpers_for_parachain!(AssetHubWestend);
impl_foreign_assets_helpers_for_parachain!(AssetHubWestend, xcm::v5::Location);
impl_xcm_helpers_for_parachain!(AssetHubWestend);
impl_bridge_helpers_for_chain!(
	AssetHubWestend,
	ParaPallet,
	PolkadotXcm,
	bp_bridge_hub_westend::RuntimeCall::XcmOverBridgeHubRococo
);
