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

#![cfg_attr(not(feature = "std"), no_std)]

use super::{Config, Round, Weight};
use frame_support::traits::{Get, OnRuntimeUpgrade};
use sp_std::marker::PhantomData;

pub fn update_for_async<T: Config>() -> Weight {
	if let Some(mut round) = <Round<T>>::get() {
		round.length = 3000u32;
		<Round<T>>::put(round);
	}

	T::DbWeight::get().reads(1) + T::DbWeight::get().writes(1)
}

pub struct SystemStakingOnRuntimeUpgrade<T>(PhantomData<T>);
impl<T: Config> OnRuntimeUpgrade for SystemStakingOnRuntimeUpgrade<T> {
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<sp_std::prelude::Vec<u8>, sp_runtime::DispatchError> {
		#[allow(unused_imports)]
		use frame_support::{migration, Identity};
		log::info!("Bifrost `pre_upgrade`...");

		if let Some(round) = <Round<T>>::get() {
			log::info!("Old round is {:?}", round);
			assert_eq!(round.length, 1500u32);
		}

		Ok(sp_std::prelude::Vec::new())
	}

	fn on_runtime_upgrade() -> Weight {
		log::info!("Bifrost `on_runtime_upgrade`...");

		let weight = update_for_async::<T>();

		log::info!("Bifrost `on_runtime_upgrade finished`");

		weight
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_: sp_std::prelude::Vec<u8>) -> Result<(), sp_runtime::DispatchError> {
		#[allow(unused_imports)]
		use frame_support::{migration, Identity};
		log::info!("Bifrost `post_upgrade`...");

		if let Some(round) = <Round<T>>::get() {
			log::info!("New round is {:?}", round);
			assert_eq!(round.length, 3000u32);
		}

		Ok(())
	}
}
