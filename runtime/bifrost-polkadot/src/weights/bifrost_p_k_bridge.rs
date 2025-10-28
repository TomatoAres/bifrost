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

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

pub struct BifrostWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> bifrost_p_k_bridge::WeightInfo for BifrostWeight<T> {
	fn set_bridge_config() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `240`
		//  Estimated: `4764`
		// Minimum execution time: 82_957_000 picoseconds.
		Weight::from_parts(86_401_000, 0)
			.saturating_add(Weight::from_parts(0, 4764))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(3))
	}

	fn set_xcm_fee_params() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `240`
		//  Estimated: `4764`
		// Minimum execution time: 82_957_000 picoseconds.
		Weight::from_parts(86_401_000, 0)
			.saturating_add(Weight::from_parts(0, 4764))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(3))
	}

	fn transfer_out() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `240`
		//  Estimated: `4764`
		// Minimum execution time: 82_957_000 picoseconds.
		Weight::from_parts(86_401_000, 0)
			.saturating_add(Weight::from_parts(0, 4764))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(3))
	}

	fn transfer_in() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `240`
		//  Estimated: `4764`
		// Minimum execution time: 82_957_000 picoseconds.
		Weight::from_parts(86_401_000, 0)
			.saturating_add(Weight::from_parts(0, 4764))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(3))
	}
}
