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
use polkadot_primitives::runtime_api::runtime_decl_for_parachain_host::ParachainHostV13;
pub use rococo_runtime;

pub mod genesis;

// Cumulus
use emulated_integration_tests_common::{
	impl_accounts_helpers_for_relay_chain, impl_assert_events_helpers_for_relay_chain,
	impl_hrmp_channels_helpers_for_relay_chain, impl_send_transact_helpers_for_relay_chain,
	xcm_emulator::decl_test_relay_chains,
};

// Rococo declaration
decl_test_relay_chains! {
	#[api_version(12)]
	pub struct Rococo {
		genesis = genesis::genesis(),
		on_init = (),
		runtime = rococo_runtime,
		core = {
			SovereignAccountOf: rococo_runtime::xcm_config::LocationConverter,
		},
		pallets = {
			XcmPallet: rococo_runtime::XcmPallet,
			Sudo: rococo_runtime::Sudo,
			Balances: rococo_runtime::Balances,
			Hrmp: rococo_runtime::Hrmp,
			Identity: rococo_runtime::Identity,
			IdentityMigrator: rococo_runtime::IdentityMigrator,
			Treasury: rococo_runtime::Treasury,
			AssetRate: rococo_runtime::AssetRate,
		}
	},
}

// Rococo implementation
impl_accounts_helpers_for_relay_chain!(Rococo);
impl_assert_events_helpers_for_relay_chain!(Rococo);
impl_hrmp_channels_helpers_for_relay_chain!(Rococo);
impl_send_transact_helpers_for_relay_chain!(Rococo);
