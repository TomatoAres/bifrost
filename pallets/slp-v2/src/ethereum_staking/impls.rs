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

use crate::{
	common::types::{Delegator, Ledger, StakingProtocol},
	ethereum_staking::types::EthereumStaking,
	Config, Error, Event, LedgerByStakingProtocolAndDelegator, Pallet,
};
use frame_support::dispatch::DispatchResultWithPostInfo;

pub const ETHEREUM_STAKING: StakingProtocol = StakingProtocol::EthereumStaking;

impl<T: Config> Pallet<T> {
	pub fn do_ethereum_staking(
		delegator: Delegator<T::AccountId>,
		task: EthereumStaking,
	) -> DispatchResultWithPostInfo {
		Self::ensure_delegator_exist(&ETHEREUM_STAKING, &delegator)?;
		LedgerByStakingProtocolAndDelegator::<T>::mutate(
			ETHEREUM_STAKING,
			delegator.clone(),
			|ledger| -> Result<(), Error<T>> {
				if let Some(Ledger::EthereumStaking(mut pending_ledger)) = ledger.clone() {
					match task {
						EthereumStaking::Stake(amount) => {
							pending_ledger.add_lock_amount(amount);
						}
						EthereumStaking::Unstake(amount) => {
							if pending_ledger.locked < amount {
								return Err(Error::<T>::InvalidParameter);
							}
							pending_ledger.subtract_lock_amount(amount);
						}
					}
					*ledger = Some(Ledger::EthereumStaking(pending_ledger));
				};
				Ok(())
			},
		)?;
		Self::deposit_event(Event::EthereumStaking { delegator, task });
		Ok(().into())
	}
}
