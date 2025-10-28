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
use bifrost_primitives::{LocalBncLocation, BNC};

#[test]
fn reserve_transfer_dot_from_relay_to_bifrost() {
	// Init values for Relay
	let destination = Polkadot::child_location_of(BifrostPolkadot::para_id());
	let sender = PolkadotSender::get();
	let amount_to_send: Balance = 10_000_000_000;

	// Init values for Parachain
	let receiver = BifrostPolkadotReceiver::get();

	BifrostPolkadot::execute_with(|| {
		type AssetRegistry = <BifrostPolkadot as BifrostPolkadotPallet>::AssetRegistry;
		assert_ok!(AssetRegistry::force_set_location(
			<BifrostPolkadot as Chain>::RuntimeOrigin::root(),
			DOT,
			Box::new(Parent.into()),
			Weight::MAX
		));
	});

	Polkadot::execute_with(|| {
		Dmp::make_parachain_reachable(BifrostPolkadot::para_id());
		assert_ok!(
			<Polkadot as PolkadotPallet>::XcmPallet::transfer_assets_using_type_and_then(
				<Polkadot as Chain>::RuntimeOrigin::signed(sender.clone()),
				bx!(destination.clone().into()),
				bx!((Here, amount_to_send).into()),
				bx!(TransferType::LocalReserve),
				bx!(Here.into()),
				bx!(TransferType::LocalReserve),
				bx!(VersionedXcm::from(
					Xcm::<()>::builder_unsafe()
						.deposit_asset(AllCounted(1), receiver.clone())
						.build()
				)),
				WeightLimit::Unlimited,
			)
		);
	});

	// Assert DOT is received on Parachain
	BifrostPolkadot::execute_with(|| {
		type RuntimeEvent = <BifrostPolkadot as Chain>::RuntimeEvent;
		assert_expected_events!(
			BifrostPolkadot,
			vec![
				RuntimeEvent::Tokens(orml_tokens::Event::Deposited { currency_id, who , amount }) => {
					currency_id: *currency_id == DOT,
					who: *who == receiver,
					amount: *amount == amount_to_send - 30500106,
				},
			]
		);
	});
}

#[test]
fn reserve_transfer_dot_from_asset_hub_to_bifrost() {
	// Init values for Relay
	let destination = AssetHubPolkadot::sibling_location_of(BifrostPolkadot::para_id());
	let sender = PolkadotSender::get();
	let amount_to_send: Balance = 10_000_000_000;

	// Init values for Parachain
	let receiver = BifrostPolkadotReceiver::get();

	BifrostPolkadot::execute_with(|| {
		type AssetRegistry = <BifrostPolkadot as BifrostPolkadotPallet>::AssetRegistry;
		assert_ok!(AssetRegistry::force_set_location(
			<BifrostPolkadot as Chain>::RuntimeOrigin::root(),
			DOT,
			Box::new(Parent.into()),
			Weight::MAX
		));
	});

	AssetHubPolkadot::execute_with(|| {
		assert_ok!(
			<AssetHubPolkadot as AssetHubPolkadotPallet>::PolkadotXcm::transfer_assets_using_type_and_then(
				<AssetHubPolkadot as Chain>::RuntimeOrigin::signed(sender.clone()),
				bx!(destination.clone().into()),
				bx!((Parent, amount_to_send).into()),
				bx!(TransferType::LocalReserve),
				bx!(Parent.into()),
				bx!(TransferType::LocalReserve),
				bx!(VersionedXcm::from(
					Xcm::<()>::builder_unsafe()
						.deposit_asset(AllCounted(1), receiver.clone())
						.build()
				)),
				WeightLimit::Unlimited,
			)
		);
	});

	BifrostPolkadot::execute_with(|| {
		type RuntimeEvent = <BifrostPolkadot as Chain>::RuntimeEvent;
		assert_expected_events!(
			BifrostPolkadot,
			vec![
				RuntimeEvent::Tokens(orml_tokens::Event::Deposited { currency_id, who , amount }) => {
					currency_id: *currency_id == DOT,
					who: *who == receiver,
					amount: *amount == amount_to_send - 30500106,
				},
			]
		);
	});
}

#[test]
fn reserve_transfer_dot_from_bifrost_to_asset_hub() {
	// Init values for Relay
	let destination = BifrostPolkadot::sibling_location_of(AssetHubPolkadot::para_id());
	let sender = BifrostPolkadotSender::get();
	let amount_to_send: Balance = 10_000_000_000;

	// Init values for Parachain
	let receiver = BifrostPolkadotReceiver::get();

	BifrostPolkadot::execute_with(|| {
		type AssetRegistry = <BifrostPolkadot as BifrostPolkadotPallet>::AssetRegistry;
		assert_ok!(AssetRegistry::force_set_location(
			<BifrostPolkadot as Chain>::RuntimeOrigin::root(),
			DOT,
			Box::new(Parent.into()),
			Weight::MAX
		));
	});

	BifrostPolkadot::execute_with(|| {
		assert_ok!(
			<BifrostPolkadot as BifrostPolkadotPallet>::PolkadotXcm::transfer_assets_using_type_and_then(
				<BifrostPolkadot as Chain>::RuntimeOrigin::signed(sender.clone()),
				bx!(destination.clone().into()),
				bx!((Parent, amount_to_send).into()),
				bx!(TransferType::DestinationReserve),
				bx!(Parent.into()),
				bx!(TransferType::DestinationReserve),
				bx!(VersionedXcm::from(
					Xcm::<()>::builder_unsafe()
						.deposit_asset(AllCounted(1), receiver.clone())
						.build()
				)),
				WeightLimit::Unlimited,
			)
		);
	});
}

#[test]
fn teleport_transfer_bnc_from_bifrost_to_asset_hub() {
	// Init values for Relay
	let destination = BifrostPolkadot::sibling_location_of(AssetHubPolkadot::para_id());
	let sender = BifrostPolkadotSender::get();
	let amount_to_send: Balance = 10_000_000_000_000;

	// Init values for Parachain
	let receiver = BifrostPolkadotReceiver::get();

	BifrostPolkadot::execute_with(|| {
		type AssetRegistry = <BifrostPolkadot as BifrostPolkadotPallet>::AssetRegistry;
		assert_ok!(AssetRegistry::force_set_location(
			<BifrostPolkadot as Chain>::RuntimeOrigin::root(),
			BNC,
			Box::new(LocalBncLocation::get().into()),
			Weight::MAX
		));
	});

	BifrostPolkadot::execute_with(|| {
		assert_ok!(
			<BifrostPolkadot as BifrostPolkadotPallet>::PolkadotXcm::transfer_assets_using_type_and_then(
				<BifrostPolkadot as Chain>::RuntimeOrigin::signed(sender.clone()),
				bx!(destination.clone().into()),
				bx!((LocalBncLocation::get(), amount_to_send).into()),
				bx!(TransferType::Teleport),
				bx!(LocalBncLocation::get().into()),
				bx!(TransferType::Teleport),
				bx!(VersionedXcm::from(
					Xcm::<()>::builder_unsafe()
						.deposit_asset(AllCounted(1), receiver.clone())
						.build()
				)),
				WeightLimit::Unlimited,
			)
		);
	});
}

#[test]
fn teleport_transfer_bnc_with_fee_dot_from_bifrost_to_asset_hub() {
	// Init values for Relay
	let destination = BifrostPolkadot::sibling_location_of(AssetHubPolkadot::para_id());
	let sender = BifrostPolkadotSender::get();
	let amount_to_send: Balance = 10_000_000_000_000;
	let fee_to_send: Balance = 10_000_000_000;

	let assets: Assets = vec![
		(Parent, fee_to_send).into(),
		(LocalBncLocation::get(), amount_to_send).into(),
	]
	.into();

	// Init values for Parachain
	let receiver = BifrostPolkadotReceiver::get();

	BifrostPolkadot::execute_with(|| {
		type AssetRegistry = <BifrostPolkadot as BifrostPolkadotPallet>::AssetRegistry;
		assert_ok!(AssetRegistry::force_set_location(
			<BifrostPolkadot as Chain>::RuntimeOrigin::root(),
			BNC,
			Box::new(LocalBncLocation::get().into()),
			Weight::MAX
		));
		assert_ok!(AssetRegistry::force_set_location(
			<BifrostPolkadot as Chain>::RuntimeOrigin::root(),
			DOT,
			Box::new(Parent.into()),
			Weight::MAX
		));
	});

	BifrostPolkadot::execute_with(|| {
		assert_ok!(
			<BifrostPolkadot as BifrostPolkadotPallet>::PolkadotXcm::transfer_assets_using_type_and_then(
				<BifrostPolkadot as Chain>::RuntimeOrigin::signed(sender.clone()),
				bx!(destination.clone().into()),
				bx!(assets.into()),
				bx!(TransferType::Teleport),
				bx!(Parent.into()),
				bx!(TransferType::DestinationReserve),
				bx!(VersionedXcm::from(
					Xcm::<()>::builder_unsafe()
						.deposit_asset(AllCounted(2), receiver.clone())
						.build()
				)),
				WeightLimit::Unlimited,
			)
		);
	});
}
