#![cfg(feature = "runtime-benchmarks")]

use crate::{types::*, *};
use bifrost_primitives::BNC;
use frame_benchmarking::v2::*;
use frame_support::BoundedVec;
use frame_system::RawOrigin as SystemOrigin;
use frame_system::RawOrigin;
use ismp::host::StateMachine;
use scale_info::prelude::collections::BTreeMap;
use sp_runtime::traits::StaticLookup;
use sp_runtime::traits::UniqueSaturatedFrom;
use token_gateway_primitives::{GatewayAssetRegistration, GatewayAssetUpdate};

#[benchmarks(where T: Config + pallet_balances::Config<Balance = u128>)]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn create_erc6160_asset() -> Result<(), BenchmarkError> {
		let account: T::AccountId = whitelisted_caller();

		let asset_details = GatewayAssetRegistration {
			name: BoundedVec::try_from(b"Spectre".to_vec()).unwrap(),
			symbol: BoundedVec::try_from(b"SPC".to_vec()).unwrap(),
			chains: vec![StateMachine::Evm(100)],
			minimum_balance: Some(10),
		};

		let mut precision = BTreeMap::new();
		for i in 0..18 {
			precision.insert(StateMachine::Evm(i as u32), 18);
		}

		let asset = AssetRegistration {
			local_id: BNC,
			reg: asset_details,
		};

		let account_id = T::Lookup::unlookup(account.clone());
		let _ = pallet_balances::Pallet::<T>::force_set_balance(
			SystemOrigin::Root.into(),
			account_id,
			10_000_000_000_000u128,
		);

		#[extrinsic_call]
		_(RawOrigin::Signed(account), asset, true);

		Ok(())
	}

	#[benchmark]
	fn teleport() -> Result<(), BenchmarkError> {
		let account: T::AccountId = whitelisted_caller();

		let asset_id = BNC;

		Pallet::<T>::create_erc6160_asset(
			RawOrigin::Signed(account.clone()).into(),
			AssetRegistration {
				local_id: asset_id,
				reg: GatewayAssetRegistration {
					name: BoundedVec::try_from(b"Spectre".to_vec()).unwrap(),
					symbol: BoundedVec::try_from(b"SPC".to_vec()).unwrap(),
					chains: vec![StateMachine::Evm(100)],
					minimum_balance: None,
				},
			},
			true,
		)?;

		// let _ = T::NativeCurrency::deposit_creating(&account, u128::MAX.into());
		let teleport_params = TeleportParams {
			asset_id,
			destination: StateMachine::Evm(100),
			recepient: H256::from([1u8; 32]),
			amount: BalanceOf::<T>::unique_saturated_from(10_000_000_000_000u128),
			timeout: 0,
			token_gateway: vec![1, 2, 3, 4, 5],
			relayer_fee: BalanceOf::<T>::unique_saturated_from(0u128),
			call_data: None,
			redeem: false,
		};

		#[extrinsic_call]
		_(RawOrigin::Signed(account), teleport_params);
		Ok(())
	}

	#[benchmark]
	fn set_token_gateway_addresses(x: Linear<1, 100>) -> Result<(), BenchmarkError> {
		let account: T::AccountId = whitelisted_caller();

		let mut addresses = BTreeMap::new();
		for i in 0..x {
			let addr = i.to_string().as_bytes().to_vec();
			addresses.insert(StateMachine::Evm(100), addr);
		}

		#[extrinsic_call]
		_(RawOrigin::Signed(account), addresses);
		Ok(())
	}

	#[benchmark]
	fn update_erc6160_asset() -> Result<(), BenchmarkError> {
		let account: T::AccountId = whitelisted_caller();

		let local_id = BNC;

		Pallet::<T>::create_erc6160_asset(
			RawOrigin::Signed(account.clone()).into(),
			AssetRegistration {
				local_id,
				reg: GatewayAssetRegistration {
					name: BoundedVec::try_from(b"Spectre".to_vec()).unwrap(),
					symbol: BoundedVec::try_from(b"SPC".to_vec()).unwrap(),
					chains: vec![StateMachine::Evm(100)],
					minimum_balance: None,
				},
			},
			true,
		)?;

		let asset_update = GatewayAssetUpdate {
			asset_id: sp_io::hashing::keccak_256(b"SPC".as_ref()).into(),
			add_chains: BoundedVec::try_from(vec![StateMachine::Evm(200)]).unwrap(),
			remove_chains: BoundedVec::try_from(Vec::new()).unwrap(),
			new_admins: BoundedVec::try_from(Vec::new()).unwrap(),
		};

		#[extrinsic_call]
		_(RawOrigin::Signed(account), asset_update);
		Ok(())
	}
}
