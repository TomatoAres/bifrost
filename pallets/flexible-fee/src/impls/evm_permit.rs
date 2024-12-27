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

use crate::Error;
use bifrost_primitives::EvmPermit;
use frame_support::dispatch::{DispatchResult, DispatchResultWithPostInfo};
use sp_core::{H160, H256, U256};
use sp_std::{marker::PhantomData, vec::Vec};
use xcm::prelude::Weight;

pub struct DisabledEvmPermitHandler<T>(PhantomData<T>);
impl<T: crate::Config> EvmPermit for DisabledEvmPermitHandler<T> {
	fn validate_permit(
		_source: H160,
		_target: H160,
		_data: Vec<u8>,
		_value: U256,
		_gas_limit: u64,
		_deadline: U256,
		_v: u8,
		_r: H256,
		_s: H256,
	) -> DispatchResult {
		Err(Error::<T>::EvmPermitCallExecutionError.into())
	}

	fn dispatch_permit(
		_source: H160,
		_target: H160,
		_data: Vec<u8>,
		_value: U256,
		_gas_limit: u64,
		_max_fee_per_gas: U256,
		_max_priority_fee_per_gas: Option<U256>,
		_nonce: Option<U256>,
		_access_list: Vec<(H160, Vec<H256>)>,
	) -> DispatchResultWithPostInfo {
		Err(Error::<T>::EvmPermitCallExecutionError.into())
	}

	fn gas_price() -> (U256, Weight) {
		Default::default()
	}

	fn dispatch_weight(_gas_limit: u64) -> Weight {
		Weight::MAX
	}

	fn permit_nonce(_account: H160) -> U256 {
		U256::default()
	}

	fn on_dispatch_permit_error() {}
}
