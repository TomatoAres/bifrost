// Copyright (C) 2020-2024  Intergalactic, Limited (GIB).
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use sp_std::prelude::*;

#[benchmarks(where T::AccountId: AsRef<[u8; 32]> + frame_support::pallet_prelude::IsType<AccountId32>)]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn bind_evm_address() -> Result<(), BenchmarkError> {
		let user: T::AccountId = account("user", 0, 1);
		let evm_address = Pallet::<T>::evm_address(&user);
		assert!(!AccountExtension::<T>::contains_key(evm_address));

		#[extrinsic_call]
		_(RawOrigin::Signed(user.clone()));

		assert!(AccountExtension::<T>::contains_key(evm_address));
		Ok(())
	}

	#[benchmark]
	fn add_contract_deployer() -> Result<(), BenchmarkError> {
		let user: T::AccountId = account("user", 0, 1);
		let evm_address = Pallet::<T>::evm_address(&user);
		assert!(!ContractDeployer::<T>::contains_key(evm_address));

		#[extrinsic_call]
		_(RawOrigin::Root, evm_address);

		assert!(ContractDeployer::<T>::contains_key(evm_address));
		Ok(())
	}

	#[benchmark]
	fn remove_contract_deployer() -> Result<(), BenchmarkError> {
		let user: T::AccountId = account("user", 0, 1);
		let evm_address = Pallet::<T>::evm_address(&user);

		Pallet::<T>::add_contract_deployer(RawOrigin::Root.into(), evm_address)?;
		assert!(ContractDeployer::<T>::contains_key(evm_address));

		#[extrinsic_call]
		_(RawOrigin::Root, evm_address);

		assert!(!ContractDeployer::<T>::contains_key(evm_address));
		Ok(())
	}

	#[benchmark]
	fn renounce_contract_deployer() -> Result<(), BenchmarkError> {
		let user: T::AccountId = account("user", 0, 1);
		let evm_address = Pallet::<T>::evm_address(&user);

		Pallet::<T>::add_contract_deployer(RawOrigin::Root.into(), evm_address)?;
		Pallet::<T>::bind_evm_address(RawOrigin::Signed(user.clone()).into())?;

		assert!(ContractDeployer::<T>::contains_key(evm_address));

		#[extrinsic_call]
		_(RawOrigin::Signed(user));

		assert!(!ContractDeployer::<T>::contains_key(evm_address));
		Ok(())
	}

	impl_benchmark_test_suite!(
		Pallet,
		crate::mock::new_test_ext_benchmark(),
		crate::mock::Test
	);
}
