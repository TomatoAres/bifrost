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

pub use penpal_runtime::{self, xcm_config::RelayNetworkId as PenpalRelayNetworkId};

mod genesis;
pub use genesis::{genesis, PenpalAssetOwner, PenpalSudoAccount, ED, PARA_ID_A, PARA_ID_B};

// Substrate
use frame_support::traits::OnInitialize;
use sp_core::Encode;

// Cumulus
use emulated_integration_tests_common::{
	impl_accounts_helpers_for_parachain, impl_assert_events_helpers_for_parachain,
	impl_assets_helpers_for_parachain, impl_foreign_assets_helpers_for_parachain,
	impl_xcm_helpers_for_parachain,
	impls::{NetworkId, Parachain},
	xcm_emulator::decl_test_parachains,
};

// Polkadot
use xcm::latest::{ROCOCO_GENESIS_HASH, WESTEND_GENESIS_HASH};

// Penpal Parachain declaration
decl_test_parachains! {
	pub struct PenpalA {
		genesis = genesis(PARA_ID_A),
		on_init = {
			penpal_runtime::AuraExt::on_initialize(1);
			frame_support::assert_ok!(penpal_runtime::System::set_storage(
				penpal_runtime::RuntimeOrigin::root(),
				vec![(PenpalRelayNetworkId::key().to_vec(), NetworkId::ByGenesis(ROCOCO_GENESIS_HASH).encode())],
			));
		},
		runtime = penpal_runtime,
		core = {
			XcmpMessageHandler: penpal_runtime::XcmpQueue,
			LocationToAccountId: penpal_runtime::xcm_config::LocationToAccountId,
			ParachainInfo: penpal_runtime::ParachainInfo,
			MessageOrigin: cumulus_primitives_core::AggregateMessageOrigin,
		},
		pallets = {
			System: penpal_runtime::System,
			PolkadotXcm: penpal_runtime::PolkadotXcm,
			Assets: penpal_runtime::Assets,
			ForeignAssets: penpal_runtime::ForeignAssets,
			AssetConversion: penpal_runtime::AssetConversion,
			Balances: penpal_runtime::Balances,
		}
	},
	pub struct PenpalB {
		genesis = genesis(PARA_ID_B),
		on_init = {
			penpal_runtime::AuraExt::on_initialize(1);
			frame_support::assert_ok!(penpal_runtime::System::set_storage(
				penpal_runtime::RuntimeOrigin::root(),
				vec![(PenpalRelayNetworkId::key().to_vec(), NetworkId::ByGenesis(WESTEND_GENESIS_HASH).encode())],
			));
		},
		runtime = penpal_runtime,
		core = {
			XcmpMessageHandler: penpal_runtime::XcmpQueue,
			LocationToAccountId: penpal_runtime::xcm_config::LocationToAccountId,
			ParachainInfo: penpal_runtime::ParachainInfo,
			MessageOrigin: cumulus_primitives_core::AggregateMessageOrigin,
		},
		pallets = {
			System: penpal_runtime::System,
			PolkadotXcm: penpal_runtime::PolkadotXcm,
			Assets: penpal_runtime::Assets,
			ForeignAssets: penpal_runtime::ForeignAssets,
			AssetConversion: penpal_runtime::AssetConversion,
			Balances: penpal_runtime::Balances,
		}
	},
}

// Penpal implementation
impl_accounts_helpers_for_parachain!(PenpalA);
impl_accounts_helpers_for_parachain!(PenpalB);
impl_assert_events_helpers_for_parachain!(PenpalA);
impl_assert_events_helpers_for_parachain!(PenpalB);
impl_assets_helpers_for_parachain!(PenpalA);
impl_foreign_assets_helpers_for_parachain!(PenpalA, xcm::latest::Location);
impl_assets_helpers_for_parachain!(PenpalB);
impl_foreign_assets_helpers_for_parachain!(PenpalB, xcm::latest::Location);
impl_xcm_helpers_for_parachain!(PenpalA);
impl_xcm_helpers_for_parachain!(PenpalB);
