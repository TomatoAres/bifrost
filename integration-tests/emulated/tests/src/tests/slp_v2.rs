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
use bifrost_primitives::TimeUnit;
use bifrost_primitives::{AccountId, LocalBncLocation, BNC};
use bifrost_slp_v2::common::types::Delegator;
use bifrost_slp_v2::common::types::ProtocolConfiguration;
use bifrost_slp_v2::common::types::StakingProtocol;
use bifrost_slp_v2::common::types::XcmFee;
use frame_support::traits::ConstU32;
use sp_core::crypto::Ss58Codec;
use sp_runtime::Permill;

#[test]
fn slp_v2_transfer_to() {
	let protocol = StakingProtocol::GeneralXCMStaking(DOT, 1000u32);
	let delegator = Delegator::Substrate(
		AccountId::from_ss58check("13PwrD5KB2eHd8ebBWiv8sCF38pj7FN9Nda6chQ9j3y6HXJE")
			.unwrap()
			.into(),
	);

	BifrostPolkadot::execute_with(|| {
		type AssetRegistry = <BifrostPolkadot as BifrostPolkadotPallet>::AssetRegistry;
		assert_ok!(AssetRegistry::force_set_location(
			<BifrostPolkadot as Chain>::RuntimeOrigin::root(),
			DOT,
			Box::new(Parent.into()),
			Weight::MAX
		));
		assert_ok!(
			<BifrostPolkadot as BifrostPolkadotPallet>::Tokens::set_balance(
				<BifrostPolkadot as Chain>::RuntimeOrigin::root(),
				AccountId::from_ss58check("5EYCAe5fv9kYSUjA6XNiMiDbzbnSH7B1KiEceQDLeYSAcJUo")
					.unwrap()
					.into(),
				DOT,
				1_000_000_000_000_000u128,
				1_000_000_000_000_000u128,
			)
		);
	});

	BifrostPolkadot::execute_with(|| {
		type SlpV2 = <BifrostPolkadot as BifrostPolkadotPallet>::SlpV2;
		assert_ok!(SlpV2::add_delegator(
			<BifrostPolkadot as Chain>::RuntimeOrigin::root(),
			protocol,
			None,
		));

		assert_ok!(SlpV2::transfer_to(
			<BifrostPolkadot as Chain>::RuntimeOrigin::root(),
			protocol,
			delegator,
			Some(DOT),
			None,
			None,
			None,
		));
	});
}

#[test]
fn slp_v2_general_xcm_executor() {
	let protocol = StakingProtocol::GeneralXCMStaking(DOT, 1000u32);
	let mut add_heads: BoundedVec<BoundedVec<u8, ConstU32<32>>, ConstU32<64>> =
		BoundedVec::default();
	let head = vec![0u8].try_into().unwrap();
	add_heads.try_push(head).unwrap();

	let value: u32 = 0x00000420;
	let bytes = value.to_be_bytes();
	let call_data: BoundedVec<u8, ConstU32<32>> = bytes.to_vec().try_into().unwrap();

	let config: ProtocolConfiguration<AccountId> = ProtocolConfiguration {
		xcm_task_fee: XcmFee {
			weight: Weight::zero(),
			fee: 100,
		},
		protocol_fee_rate: Permill::from_perthousand(100),
		unlock_period: TimeUnit::Era(9),
		operator: AccountId::new([0u8; 32]),
		max_update_token_exchange_rate: Permill::from_perthousand(1),
		update_time_unit_interval: 100u32,
		update_exchange_rate_interval: 100u32,
		remote_fee_location: Some(Location::here()),
	};

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
		assert_ok!(
			<BifrostPolkadot as BifrostPolkadotPallet>::SlpV2::set_protocol_configuration(
				<BifrostPolkadot as Chain>::RuntimeOrigin::root(),
				protocol,
				config
			)
		);
		assert_ok!(
			<BifrostPolkadot as BifrostPolkadotPallet>::Balances::force_set_balance(
				<BifrostPolkadot as Chain>::RuntimeOrigin::root(),
				AccountId::from_ss58check("5EYCAe5fv9kYSUjA6XNiMiDbzbnSH7B1KiEceQDLeYSAcJUo")
					.unwrap()
					.into(),
				1_000_000_000_000_000u128,
			)
		);
		assert_ok!(
			<BifrostPolkadot as BifrostPolkadotPallet>::Tokens::set_balance(
				<BifrostPolkadot as Chain>::RuntimeOrigin::root(),
				AccountId::from_ss58check("5EYCAe5fv9kYSUjA6XNiMiDbzbnSH7B1KiEceQDLeYSAcJUo")
					.unwrap()
					.into(),
				DOT,
				1_000_000_000_000_000u128,
				1_000_000_000_000_000u128,
			)
		);
	});

	AssetHubPolkadot::execute_with(|| {
		assert_ok!(
			<AssetHubPolkadot as AssetHubPolkadotPallet>::Balances::force_set_balance(
				<AssetHubPolkadot as Chain>::RuntimeOrigin::root(),
				AccountId::from_ss58check("13cKp89TtYknbyYnqnF6dWN75q5ZosvFSuqzoEVkUAaNR47A")
					.unwrap()
					.into(),
				1_000_000_000_000_000u128,
			)
		);

		assert_ok!(
			<AssetHubPolkadot as AssetHubPolkadotPallet>::Balances::force_set_balance(
				<AssetHubPolkadot as Chain>::RuntimeOrigin::root(),
				AccountId::from_ss58check("13PwrD5KB2eHd8ebBWiv8sCF38pj7FN9Nda6chQ9j3y6HXJE")
					.unwrap()
					.into(),
				1_000_000_000_000_000u128,
			)
		);
	});

	BifrostPolkadot::execute_with(|| {
		type SlpV2 = <BifrostPolkadot as BifrostPolkadotPallet>::SlpV2;
		assert_ok!(SlpV2::add_delegator(
			<BifrostPolkadot as Chain>::RuntimeOrigin::root(),
			protocol,
			None,
		));

		assert_ok!(SlpV2::update_xcm_executor_whitelist(
			<BifrostPolkadot as Chain>::RuntimeOrigin::root(),
			DOT,
			1000u32,
			Some(add_heads),
			None,
		));

		assert_ok!(SlpV2::general_xcm_executor(
			<BifrostPolkadot as Chain>::RuntimeOrigin::root(),
			DOT,
			1000u32,
			call_data,
		));
	});
}
