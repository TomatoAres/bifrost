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

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::Pallet as AssetRegistry;
use bifrost_primitives::CurrencyId;
use frame_benchmarking::v2::*;
use frame_support::{assert_ok, traits::UnfilteredDispatchable};
use sp_runtime::traits::UniqueSaturatedFrom;

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn register_token_metadata() -> Result<(), BenchmarkError> {
		let origin =
			T::RegisterOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;
		let metadata = AssetMetadata {
			name: b"Bifrost Native Coin".to_vec(),
			symbol: b"BNC".to_vec(),
			decimals: 12,
			minimal_balance: BalanceOf::<T>::unique_saturated_from(0u128),
		};

		let call = Call::<T>::register_token_metadata {
			metadata: Box::new(metadata.clone()),
		};

		#[block]
		{
			call.dispatch_bypass_filter(origin)?;
		}

		assert_eq!(
			CurrencyMetadatas::<T>::get(Token2(0)),
			Some(metadata.clone())
		);
		Ok(())
	}

	#[benchmark]
	fn register_vtoken_metadata() -> Result<(), BenchmarkError> {
		let origin =
			T::RegisterOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;
		let metadata = AssetMetadata {
			name: b"Bifrost Native Coin".to_vec(),
			symbol: b"BNC".to_vec(),
			decimals: 12,
			minimal_balance: BalanceOf::<T>::unique_saturated_from(0u128),
		};
		let v_metadata = AssetMetadata {
			name: b"Voucher BNC".to_vec(),
			symbol: b"vBNC".to_vec(),
			decimals: 12,
			minimal_balance: BalanceOf::<T>::unique_saturated_from(0u128),
		};
		assert_ok!(AssetRegistry::<T>::register_token_metadata(
			origin.clone(),
			Box::new(metadata.clone())
		));

		let call = Call::<T>::register_vtoken_metadata { token_id: 0 };

		#[block]
		{
			call.dispatch_bypass_filter(origin)?;
		}

		assert_eq!(
			CurrencyMetadatas::<T>::get(CurrencyId::VToken2(0)),
			Some(v_metadata.clone())
		);
		Ok(())
	}

	#[benchmark]
	fn register_location() -> Result<(), BenchmarkError> {
		let origin =
			T::RegisterOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;
		let metadata = AssetMetadata {
			name: b"Bifrost Native Coin".to_vec(),
			symbol: b"BNC".to_vec(),
			decimals: 12,
			minimal_balance: BalanceOf::<T>::unique_saturated_from(0u128),
		};
		let versioned_location = VersionedLocation::V4(Location::new(1, [Parachain(2001)]));
		let location: xcm::v4::Location = versioned_location.clone().try_into().unwrap();

		assert_ok!(AssetRegistry::<T>::register_token_metadata(
			origin.clone(),
			Box::new(metadata.clone())
		));

		let call = Call::<T>::register_location {
			currency_id: Token2(0),
			location: Box::new(versioned_location.clone()),
			weight: Weight::from_parts(2000_000_000, u64::MAX),
		};

		#[block]
		{
			call.dispatch_bypass_filter(origin)?;
		}

		assert_eq!(
			LocationToCurrencyIds::<T>::get(location.clone()),
			Some(Token2(0))
		);
		assert_eq!(CurrencyIdToLocations::<T>::get(Token2(0)), Some(location));
		assert_eq!(
			CurrencyIdToWeights::<T>::get(Token2(0)),
			Some(Weight::from_parts(2000_000_000, u64::MAX))
		);
		Ok(())
	}

	#[benchmark]
	fn force_set_location() -> Result<(), BenchmarkError> {
		let origin =
			T::RegisterOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;
		let metadata = AssetMetadata {
			name: b"Bifrost Native Coin".to_vec(),
			symbol: b"BNC".to_vec(),
			decimals: 12,
			minimal_balance: BalanceOf::<T>::unique_saturated_from(0u128),
		};
		let versioned_location = VersionedLocation::V4(Location::new(1, [Parachain(2001)]));

		assert_ok!(AssetRegistry::<T>::register_token_metadata(
			origin.clone(),
			Box::new(metadata.clone())
		));

		let call = Call::<T>::force_set_location {
			currency_id: Token2(0),
			location: Box::new(versioned_location.clone()),
			weight: Weight::from_parts(2000_000_000, u64::MAX),
		};

		#[block]
		{
			call.dispatch_bypass_filter(origin)?;
		}

		Ok(())
	}

	#[benchmark]
	fn update_currency_metadata() -> Result<(), BenchmarkError> {
		let origin =
			T::RegisterOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;

		assert_ok!(AssetRegistry::<T>::register_token_metadata(
			origin.clone(),
			Box::new(AssetMetadata {
				name: b"Old Token Name".to_vec(),
				symbol: b"OTN".to_vec(),
				decimals: 10,
				minimal_balance: BalanceOf::<T>::unique_saturated_from(1u128),
			})
		));

		let call = Call::<T>::update_currency_metadata {
			currency_id: CurrencyId::Token2(0),
			asset_name: Some(b"Token Name".to_vec()),
			asset_symbol: Some(b"TN".to_vec()),
			asset_decimals: Some(12),
			asset_minimal_balance: Some(BalanceOf::<T>::unique_saturated_from(1000u128)),
		};

		#[block]
		{
			call.dispatch_bypass_filter(origin)?;
		}

		assert_eq!(
			CurrencyMetadatas::<T>::get(CurrencyId::Token2(0)),
			Some(AssetMetadata {
				name: b"Token Name".to_vec(),
				symbol: b"TN".to_vec(),
				decimals: 12,
				minimal_balance: BalanceOf::<T>::unique_saturated_from(1000u128),
			})
		);
		Ok(())
	}

	// This line generates test cases for benchmarking, and could be run by:
	//   `cargo test -p pallet-example-basic --all-features`, you will see one line per case:
	//   `test benchmarking::bench_sort_vector ... ok`
	//   `test benchmarking::bench_accumulate_dummy ... ok`
	//   `test benchmarking::bench_set_dummy_benchmark ... ok` in the result.
	//
	// The line generates three steps per benchmark, with repeat=1 and the three steps are
	//   [low, mid, high] of the range.
	impl_benchmark_test_suite!(
		Pallet,
		crate::mock::new_test_ext_benchmark(),
		crate::mock::Runtime
	);
}
