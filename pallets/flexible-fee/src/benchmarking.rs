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

use frame_benchmarking::v2::*;
use frame_support::BoundedVec;

use bifrost_primitives::{CurrencyId, TokenSymbol};
use frame_system::RawOrigin;
use sp_std::vec;

use crate::{Call, Config, Pallet};

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn set_user_default_fee_currency() -> Result<(), BenchmarkError> {
		let caller = whitelisted_caller();

		#[extrinsic_call]
		_(
			RawOrigin::Signed(caller),
			Some(CurrencyId::Token(TokenSymbol::DOT)),
		);

		Ok(())
	}

	#[benchmark]
	fn set_default_fee_currency_list() -> Result<(), BenchmarkError> {
		let default_list = BoundedVec::try_from(vec![CurrencyId::Token(TokenSymbol::DOT)]).unwrap();

		#[extrinsic_call]
		_(RawOrigin::Root, default_list);

		Ok(())
	}
	impl_benchmark_test_suite!(
		Pallet,
		crate::mocks::kusama_mock::new_test_ext_benchmark(),
		crate::mocks::kusama_mock::Test
	);
}
