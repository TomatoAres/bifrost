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

use crate::imports::*;
use crate::{
	tests::{
		asset_hub_polkadot_location, bridge_hub_polkadot_location, create_foreign_on_ah_kusama,
		create_foreign_on_ah_polkadot, set_up_pool_with_dot_on_ah_polkadot,
		set_up_pool_with_ksm_on_ah_kusama,
	},
	*,
};
use bifrost_primitives::AccountId;
use bifrost_primitives::RemoteBifrostKusamaBncLocation;
use bifrost_primitives::RemoteBifrostPolkadotBncLocation;
use bifrost_primitives::DOT as DOT_CURRENCY;
use bifrost_primitives::KSM as KSM_CURRENCY;
use xcm::latest::Parent;

use crate::tests::{
	asset_hub_kusama_location, bridge_hub_kusama_location, bridged_dot_at_ah_kusama,
	bridged_ksm_at_ah_polkadot,
};
use emulated_integration_tests_common::{impls::Parachain, xcm_emulator::ConvertLocation};

pub(crate) const KSM: u128 = 1_000_000_000_000;
pub(crate) const DOT: u128 = 10_000_000_000;

fn setup_xcm_versions() {
	// For BK -> BP
	// BifrostKusama -> AssetHubKusama -> BridgeHubKusama -> BridgeHubPolkadot -> AssetHubPolkadot -> BifrostPolkadot

	// AssetHubKusama -> AssetHubPolkadot
	AssetHubKusama::force_xcm_version(asset_hub_polkadot_location(), XCM_VERSION);
	// AssetHubPolkadot -> BifrostPolkadot
	// AssetHubPolkadot::force_xcm_version(Location::new(1, [Parachain(2030)]), XCM_VERSION);
	// AssetHubPolkadot::force_xcm_version(bifrost_kusama_global_location(), XCM_VERSION);
	BridgeHubKusama::force_xcm_version(bridge_hub_polkadot_location(), XCM_VERSION);

	BifrostPolkadot::execute_with(|| {
		type AssetRegistry = <BifrostPolkadot as BifrostPolkadotPallet>::AssetRegistry;
		assert_ok!(AssetRegistry::force_set_location(
			<BifrostPolkadot as Chain>::RuntimeOrigin::root(),
			DOT_CURRENCY,
			Box::new(Parent.into()),
			Weight::MAX
		));
	});

	// For BP -> BK
	AssetHubPolkadot::force_xcm_version(asset_hub_kusama_location(), XCM_VERSION);
	// AssetHubKusama::force_xcm_version(bifrost_kusama_location(), XCM_VERSION);
	// AssetHubKusama::force_xcm_version(bifrost_polkadot_global_location(), XCM_VERSION);
	BridgeHubPolkadot::force_xcm_version(bridge_hub_kusama_location(), XCM_VERSION);

	BifrostKusama::execute_with(|| {
		type AssetRegistry = <BifrostKusama as BifrostKusamaPallet>::AssetRegistry;
		assert_ok!(AssetRegistry::force_set_location(
			<BifrostKusama as Chain>::RuntimeOrigin::root(),
			KSM_CURRENCY,
			Box::new(Parent.into()),
			Weight::MAX
		));
	});
}

pub(crate) fn bk_to_bp_bridge_setup() {
	setup_xcm_versions();

	// Fund accounts

	// let bifrost_location = AssetHubKusama::sibling_location_of(BifrostKusama::para_id());
	// let sov_bifrost_on_kah = AssetHubKusama::sovereign_account_id_of(bifrost_location);
	// AssetHubKusama::fund_accounts(vec![(sov_bifrost_on_kah, 10_000_000_000_000u128)]);

	// fund the BifrostKusama's SA on AssetHubKusama for paying bridge transport fees
	AssetHubKusama::fund_para_sovereign(BifrostKusama::para_id(), 10_000_000_000_000u128);

	// fund the KAH's SA on BHR for paying bridge transport fees
	BridgeHubKusama::fund_para_sovereign(AssetHubKusama::para_id(), 10_000_000_000_000u128);

	let sov_ahp_on_ahk = AssetHubPolkadot::sovereign_account_of_parachain_on_other_global_consensus(
		Kusama,
		BifrostKusama::para_id(),
	);
	AssetHubPolkadot::fund_accounts(vec![(sov_ahp_on_ahk, 10_000_000_000_000u128)]);

	let sov_ahp_on_bk = AssetHubPolkadot::sovereign_account_of_parachain_on_other_global_consensus(
		Kusama,
		BifrostKusama::para_id(),
	);
	let sov_ahp_on_bp = AssetHubKusama::sovereign_account_of_parachain_on_other_global_consensus(
		Polkadot,
		BifrostPolkadot::para_id(),
	);
	println!("======================================={:?}", sov_ahp_on_bk);
	println!("======================================={:?}", sov_ahp_on_bp);

	create_foreign_on_ah_kusama(RemoteBifrostKusamaBncLocation::get(), false, vec![]);
	set_up_pool_with_ksm_on_ah_kusama(RemoteBifrostKusamaBncLocation::get(), true);

	let bridged_ksm_at_ah_polkadot = bridged_ksm_at_ah_polkadot();
	create_foreign_on_ah_polkadot(bridged_ksm_at_ah_polkadot.clone(), true, vec![]);
	set_up_pool_with_dot_on_ah_polkadot(bridged_ksm_at_ah_polkadot.clone(), true);
}

pub(crate) fn bp_to_bk_bridge_setup() {
	setup_xcm_versions();

	// Fund accounts

	// fund the KAH's SA on KBH for paying bridge transport fees
	BridgeHubPolkadot::fund_para_sovereign(AssetHubPolkadot::para_id(), 100 * DOT);
	// fund the BifrostKusama's SA on AssetHubKusama for paying bridge transport fees
	AssetHubPolkadot::fund_para_sovereign(BifrostPolkadot::para_id(), 100_000_000_000_000u128);

	let sov_ahp_on_ahk = AssetHubKusama::sovereign_account_of_parachain_on_other_global_consensus(
		Polkadot,
		BifrostPolkadot::para_id(),
	);
	AssetHubKusama::fund_accounts(vec![(sov_ahp_on_ahk.clone(), 100 * KSM)]);

	create_foreign_on_ah_polkadot(RemoteBifrostPolkadotBncLocation::get(), false, vec![]);
	set_up_pool_with_dot_on_ah_polkadot(RemoteBifrostPolkadotBncLocation::get(), true);

	let bridged_dot_at_ah_kusama = bridged_dot_at_ah_kusama();
	create_foreign_on_ah_kusama(bridged_dot_at_ah_kusama.clone(), true, vec![]);
	set_up_pool_with_ksm_on_ah_kusama(bridged_dot_at_ah_kusama.clone(), true);

	// create_foreign_on_ah_kusama(ik_sibling(), false, vec![]);
	// set_up_pool_with_ksm_on_ah_kusama(ik_sibling(), true);
	//
	// create_reserve_asset_on_bk(KSM_CURRENCY, Parent.into());
}
