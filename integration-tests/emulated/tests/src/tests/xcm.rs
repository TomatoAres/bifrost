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
use sp_core::hashing;
use xcm::opaque::latest::AssetTransferFilter;

#[test]
fn transfer_and_transact() {
	let destination = AssetHubPolkadot::sibling_location_of(BifrostPolkadot::para_id());
	let sender = AssetHubPolkadotSender::get();
	println!("sender: {:?}", sender);
	let amount_to_send: Balance = ASSET_HUB_POLKADOT_ED * 100000;
	let assets: Assets = (Parent, amount_to_send).into();
	let receiver = BifrostPolkadotReceiver::get();
	println!("amount_to_send: {:?}", amount_to_send);
	println!("receiver: {:?}", receiver);

	BifrostPolkadot::execute_with(|| {
		type AssetRegistry = <BifrostPolkadot as BifrostPolkadotPallet>::AssetRegistry;
		assert_ok!(AssetRegistry::force_set_location(
			<BifrostPolkadot as Chain>::RuntimeOrigin::root(),
			DOT,
			Box::new(Parent.into()),
			Weight::MAX
		));
	});

	// `WithdrawAsset` parameters.
	let assets_to_withdraw = vec![(Parent, amount_to_send).into()];

	// `PayFees` parameters.
	let fees_amount = 10000 * ASSET_HUB_POLKADOT_ED;
	let fees_assets: Asset = (Parent, fees_amount).into();
	println!("fees_assets: {:?}", fees_assets);

	// `Transact` parameters (remember this is on AssetHubWestend!).
	// How to convert the location into a FRAME origin.
	// In this case, we want a two step process where we:
	// - Convert the location into an account (the sovereign account)
	// - Convert that account into a FRAME Signed origin.
	let origin_kind = OriginKind::SovereignAccount;
	// An optional value we only need when we want backwards compatibility
	// with versions older than 5.
	let fallback_max_weight = None;
	// We want to execute the `remark_with_event` call on the asset hub.
	let remark = b"Hello, world!".to_vec();
	let remark_hash = hashing::blake2_256(&remark);
	let call = <BifrostPolkadot as Chain>::RuntimeCall::System(frame_system::Call::<
		<BifrostPolkadot as Chain>::Runtime,
	>::remark_with_event {
		remark,
	})
	.encode();

	// `InitiateTransfer` parameters.
	let remote_fees = AssetTransferFilter::ReserveDeposit(Definite((Parent, fees_amount).into()));
	// This time we NEED to preserve the origin.
	// If not, the Transact won't know how to get a FRAME origin
	// to execute the call.
	let preserve_origin = true;
	let transfer_assets = vec![AssetTransferFilter::ReserveDeposit(Wild(AllCounted(1)))];
	let remote_xcm = Xcm::<()>::builder_unsafe()
		.transact(origin_kind, fallback_max_weight, call)
		.refund_surplus()
		.deposit_asset(AllCounted(1), receiver.clone())
		.build();

	// We assemble the XCM with all the previous values.
	let xcm = Xcm::<<AssetHubPolkadot as Chain>::RuntimeCall>::builder()
		.withdraw_asset(assets_to_withdraw)
		.pay_fees(fees_assets)
		.initiate_transfer(
			destination,
			remote_fees,
			preserve_origin,
			transfer_assets,
			remote_xcm,
		)
		.build();

	// We execute the XCM and assert that the `amount_to_send` is taken
	// out of the senders account.
	AssetHubPolkadot::execute_with(|| {
		assert_ok!(
			<AssetHubPolkadot as AssetHubPolkadotPallet>::PolkadotXcm::execute(
				<AssetHubPolkadot as Chain>::RuntimeOrigin::signed(sender.clone()),
				Box::new(VersionedXcm::from(xcm)),
				Weight::MAX,
			)
		);
	});

	// We check that the event from the transaction we made is actually emitted.
	BifrostPolkadot::execute_with(|| {
		let sov_account_of_sender_on_bifrost_polkadot =
			BifrostPolkadot::sovereign_account_id_of(Location::new(
				1,
				[
					Parachain(AssetHubPolkadot::para_id().into()),
					AccountId32 {
						network: None,
						id: sender.into(),
					},
				],
			));
		println!(
			"sov_account_of_sender_on_bifrost_polkadot: {:?}",
			sov_account_of_sender_on_bifrost_polkadot
		);
		<BifrostPolkadot as BifrostPolkadotPallet>::System::assert_has_event(
			frame_system::Event::Remarked {
				sender: sov_account_of_sender_on_bifrost_polkadot,
				hash: remark_hash.into(),
			}
			.into(),
		);

		let events = <BifrostPolkadot as BifrostPolkadotPallet>::System::events();
		println!("events: {:?}", events);
	});
}

#[test]
fn transfer_and_mint() {
	let destination = AssetHubPolkadot::sibling_location_of(BifrostPolkadot::para_id());
	let sender = AssetHubPolkadotSender::get();
	let balance_of_sender = AssetHubPolkadot::execute_with(|| {
		type Balances = <AssetHubPolkadot as AssetHubPolkadotPallet>::Balances;
		Balances::free_balance(sender.clone())
	});
	println!("sender: {:?}", sender);
	println!("balance_of_sender: {:?}", balance_of_sender);
	let amount_to_send: Balance = ASSET_HUB_POLKADOT_ED * 100000;
	let assets: Assets = (Parent, amount_to_send).into();
	let receiver = BifrostPolkadotReceiver::get();
	println!("amount_to_send: {:?}", amount_to_send);
	println!("receiver: {:?}", receiver);

	BifrostPolkadot::execute_with(|| {
		type AssetRegistry = <BifrostPolkadot as BifrostPolkadotPallet>::AssetRegistry;
		assert_ok!(AssetRegistry::force_set_location(
			<BifrostPolkadot as Chain>::RuntimeOrigin::root(),
			DOT,
			Box::new(Parent.into()),
			Weight::MAX
		));
		// Set up minimum mint amount for DOT
		assert_ok!(
			<BifrostPolkadot as BifrostPolkadotPallet>::VtokenMinting::set_minimum_mint(
				<BifrostPolkadot as Chain>::RuntimeOrigin::root(),
				DOT,
				1000000000 // 1 DOT in smallest units
			)
		);

		// Set up DOT to VDOT mapping for the test
		let token_configs = vec![VTokenTokenConfig {
			token: DOT,
			redeem_enabled: true,
		}];
		assert_ok!(
			<BifrostPolkadot as BifrostPolkadotPallet>::VtokenMinting::set_vtoken_multimap(
				<BifrostPolkadot as Chain>::RuntimeOrigin::root(),
				VDOT,
				token_configs.try_into().unwrap()
			)
		);

		// Set up fees (0% for simplicity)
		assert_ok!(
			<BifrostPolkadot as BifrostPolkadotPallet>::VtokenMinting::set_fees(
				<BifrostPolkadot as Chain>::RuntimeOrigin::root(),
				sp_runtime::Permill::zero(),
				sp_runtime::Permill::zero()
			)
		);
	});
	let sov_account_of_sender_on_bifrost_polkadot =
		BifrostPolkadot::sovereign_account_id_of(Location::new(
			1,
			[
				Parachain(AssetHubPolkadot::para_id().into()),
				AccountId32 {
					network: None,
					id: sender.clone().into(),
				},
			],
		));
	// `WithdrawAsset` parameters.
	let assets_to_withdraw = vec![(Parent, amount_to_send).into()];

	// `PayFees` parameters.
	let fees_amount = 10000 * ASSET_HUB_POLKADOT_ED;
	let fees_assets: Asset = (Parent, fees_amount).into();
	println!("fees_assets: {:?}", fees_assets);

	// `Transact` parameters (remember this is on AssetHubWestend!).
	// How to convert the location into a FRAME origin.
	// In this case, we want a two step process where we:
	// - Convert the location into an account (the sovereign account)
	// - Convert that account into a FRAME Signed origin.
	let origin_kind = OriginKind::SovereignAccount;
	// An optional value we only need when we want backwards compatibility
	// with versions older than 5.
	let fallback_max_weight = None;
	let call =
		<BifrostPolkadot as Chain>::RuntimeCall::VtokenMinting(bifrost_vtoken_minting::Call::<
			<BifrostPolkadot as Chain>::Runtime,
		>::mint {
			currency_id: DOT,
			currency_amount: amount_to_send - fees_amount * 2,
			remark: BoundedVec::default(),
			channel_id: None,
		})
		.encode();

	// `InitiateTransfer` parameters.
	let remote_fees = AssetTransferFilter::ReserveDeposit(Definite((Parent, fees_amount).into()));
	// This time we NEED to preserve the origin.
	// If not, the Transact won't know how to get a FRAME origin
	// to execute the call.
	let preserve_origin = true;
	let transfer_assets = vec![AssetTransferFilter::ReserveDeposit(Wild(AllCounted(1)))];
	let remote_xcm = Xcm::<()>::builder_unsafe()
		.deposit_asset(
			AllCounted(1),
			sov_account_of_sender_on_bifrost_polkadot.clone(),
		)
		.transact(origin_kind, fallback_max_weight, call)
		// .deposit_asset(AllCounted(1), receiver.clone())
		.build();

	// We assemble the XCM with all the previous values.
	let xcm = Xcm::<<AssetHubPolkadot as Chain>::RuntimeCall>::builder()
		.withdraw_asset(assets_to_withdraw)
		.pay_fees(fees_assets)
		.initiate_transfer(
			destination,
			remote_fees,
			preserve_origin,
			transfer_assets,
			remote_xcm,
		)
		.build();

	// We execute the XCM and assert that the `amount_to_send` is taken
	// out of the senders account.
	AssetHubPolkadot::execute_with(|| {
		assert_ok!(
			<AssetHubPolkadot as AssetHubPolkadotPallet>::PolkadotXcm::execute(
				<AssetHubPolkadot as Chain>::RuntimeOrigin::signed(sender.clone()),
				Box::new(VersionedXcm::from(xcm)),
				Weight::MAX,
			)
		);
	});

	let balance_of_sender_after = AssetHubPolkadot::execute_with(|| {
		type Balances = <AssetHubPolkadot as AssetHubPolkadotPallet>::Balances;
		Balances::free_balance(sender.clone())
	});
	println!("balance_of_sender_after: {:?}", balance_of_sender_after);

	// We check that the event from the transaction we made is actually emitted.
	BifrostPolkadot::execute_with(|| {
		type RuntimeEvent = <BifrostPolkadot as Chain>::RuntimeEvent;
		let events = <BifrostPolkadot as BifrostPolkadotPallet>::System::events();
		println!("events: {:?}", events);
		assert_expected_events!(
			BifrostPolkadot,
			vec![
				// Amount to reserve transfer is transferred to Parachain's Sovereign account
				RuntimeEvent::VtokenMinting(
					bifrost_vtoken_minting::Event::Minted { minter, currency_id, .. }
				) => {
					minter: *minter == sov_account_of_sender_on_bifrost_polkadot,
					currency_id: *currency_id == DOT,
				},
			]
		);
	});
}
