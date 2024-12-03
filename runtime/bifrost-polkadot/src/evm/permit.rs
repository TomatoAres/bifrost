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

use crate::{evm::precompiles, ExtrinsicBaseWeight, Runtime};
use bifrost_primitives::{AccountId, EvmPermit};
use evm::ExitReason;
use fp_evm::FeeCalculator;
use frame_support::{
	dispatch::{DispatchErrorWithPostInfo, Pays, PostDispatchInfo, RawOrigin},
	ensure,
	pallet_prelude::DispatchResultWithPostInfo,
	traits::Time,
};
use pallet_evm::{AddressMapping, GasWeightMapping, Runner};
use pallet_evm_precompile_call_permit::NoncesStorage;
use primitive_types::{H160, H256, U256};
use sp_core::crypto::AccountId32;
use sp_io::hashing::keccak_256;
use sp_runtime::{
	traits::{One, UniqueSaturatedInto},
	DispatchResult,
};
use sp_std::vec::Vec;
use xcm::prelude::Weight;

pub struct EvmPermitHandler<R>(sp_std::marker::PhantomData<R>);

impl<R> EvmPermit for EvmPermitHandler<R>
where
	R: frame_system::Config
		+ pallet_evm::Config
		+ bifrost_flexible_fee::Config
		+ pallet_tx_pause::Config
		+ pallet_evm_accounts::Config,
	R::Nonce: Into<U256>,
	AccountId: From<R::AccountId>,
	R::AccountId: AsRef<[u8; 32]> + frame_support::traits::IsType<AccountId32>,
{
	fn validate_permit(
		source: H160,
		target: H160,
		data: Vec<u8>,
		value: U256,
		gas_limit: u64,
		deadline: U256,
		v: u8,
		r: H256,
		s: H256,
	) -> DispatchResult {
		let account_nonce = NoncesStorage::get(source);

		let permit = pallet_evm_precompile_call_permit::CallPermitPrecompile::<R>::generate_permit(
			precompiles::CALLPERMIT,
			source,
			target,
			value,
			data,
			gas_limit,
			account_nonce,
			deadline,
		);

		// Blockchain time is in ms while Ethereum use second timestamps.
		let timestamp: u128 = <R as pallet_evm::Config>::Timestamp::now().unique_saturated_into();
		let timestamp: U256 = U256::from(timestamp / 1000);

		ensure!(deadline >= timestamp, bifrost_flexible_fee::Error::<R>::EvmPermitExpired);

		let mut sig = [0u8; 65];
		sig[0..32].copy_from_slice(r.as_bytes());
		sig[32..64].copy_from_slice(s.as_bytes());
		sig[64] = v;
		let signer = sp_io::crypto::secp256k1_ecdsa_recover(&sig, &permit)
			.map_err(|_| bifrost_flexible_fee::Error::<R>::EvmPermitInvalid)?;
		let signer = H160::from(H256::from_slice(keccak_256(&signer).as_slice()));
		ensure!(
			signer != H160::zero() && signer == source,
			bifrost_flexible_fee::Error::<R>::EvmPermitInvalid
		);

		Ok(())
	}

	fn dispatch_permit(
		source: H160,
		target: H160,
		data: Vec<u8>,
		value: U256,
		gas_limit: u64,
		max_fee_per_gas: U256,
		max_priority_fee_per_gas: Option<U256>,
		_nonce: Option<U256>,
		access_list: Vec<(H160, Vec<H256>)>,
	) -> DispatchResultWithPostInfo {
		// Dispatching permit should not increase account nonce, as TX is not signed by the account.
		// Therefore, we need to manually reset it back to current value after execution.
		let account_id = <R as pallet_evm::Config>::AddressMapping::into_account_id(source);
		let source_nonce = frame_system::Account::<R>::get(&account_id).nonce;

		let is_transactional = true;
		let validate = true;
		let info = match <R as pallet_evm::Config>::Runner::call(
			source,
			target,
			data,
			value,
			gas_limit,
			Some(max_fee_per_gas),
			max_priority_fee_per_gas,
			None,
			access_list,
			is_transactional,
			validate,
			None,
			None,
			<R as pallet_evm::Config>::config(),
		) {
			Ok(info) => info,
			Err(e) =>
				return Err(DispatchErrorWithPostInfo {
					post_info: PostDispatchInfo {
						actual_weight: Some(e.weight),
						pays_fee: Pays::Yes,
					},
					error: bifrost_flexible_fee::Error::<R>::EvmPermitRunnerError.into(),
				}),
		};
		let account_source_nonce = frame_system::Account::<R>::get(&account_id).nonce;
		debug_assert_eq!(
			account_source_nonce,
			source_nonce + <R as frame_system::Config>::Nonce::one()
		);
		frame_system::Account::<R>::mutate(account_id, |a| {
			a.nonce -= <R as frame_system::Config>::Nonce::one()
		});

		let permit_nonce = NoncesStorage::get(source);
		NoncesStorage::insert(source, permit_nonce + U256::one());

		let mut gas_to_weight = <R as pallet_evm::Config>::GasWeightMapping::gas_to_weight(
			info.used_gas.standard.unique_saturated_into(),
			true,
		);
		if let Some(weight_info) = info.weight_info {
			if let Some(proof_size_usage) = weight_info.proof_size_usage {
				*gas_to_weight.proof_size_mut() = proof_size_usage;
			}
		}
		let actual_weight = gas_to_weight;
		let post_info = PostDispatchInfo { actual_weight: Some(actual_weight), pays_fee: Pays::No };

		match info.exit_reason {
			ExitReason::Succeed(_) => Ok(post_info),
			_ => Err(DispatchErrorWithPostInfo {
				post_info,
				error: bifrost_flexible_fee::Error::<R>::EvmPermitCallExecutionError.into(),
			}),
		}
	}

	fn gas_price() -> (U256, Weight) {
		<Runtime as pallet_evm::Config>::FeeCalculator::min_gas_price()
	}

	fn dispatch_weight(gas_limit: u64) -> Weight {
		let without_base_extrinsic_weight = true;
		let weight = <R as pallet_evm::Config>::GasWeightMapping::gas_to_weight(
			gas_limit,
			without_base_extrinsic_weight,
		);

		// As GasWeightMapping implementation does not include/exclude the weight-with-swap (only
		// the frame_system::constants::ExtrinsicBaseWeight) therefore we need to add it manually
		// here
		weight.saturating_add(ExtrinsicBaseWeight::get())
	}

	fn permit_nonce(account: H160) -> U256 {
		NoncesStorage::get(account)
	}

	fn on_dispatch_permit_error() {
		let _ = pallet_tx_pause::Pallet::<R>::pause(
			RawOrigin::Root.into(),
			(
				b"FlexibleFee".to_vec().try_into().expect("FlexibleFee"),
				b"dispatch_permit".to_vec().try_into().expect("FlexibleFee"),
			),
		);
	}
}
