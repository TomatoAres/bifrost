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

use crate::*;
use bifrost_primitives::{CurrencyId, DerivativeIndex};
use frame_support::pallet_prelude::*;
use parity_scale_codec::alloc::collections::BTreeMap;
use sp_runtime::traits::AccountIdConversion;
use xcm::v5::Location;

use crate::{agents::bifrost_agent::BifrostCall, pallet::Error, traits::*};

/// VotingAgent implementation for Bifrost
pub struct BifrostAgent<T: Config> {
	vtoken: CurrencyIdOf<T>,
	location: Location,
}

impl<T: Config> BifrostAgent<T> {
	// Only polkadot networks are supported.
	pub fn new(vtoken: CurrencyId) -> Result<Self, Error<T>> {
		if cfg!(feature = "polkadot") {
			let location = Pallet::<T>::convert_vtoken_to_dest_location(vtoken)?;
			Ok(Self { vtoken, location })
		} else {
			Err(Error::<T>::VTokenNotSupport)
		}
	}
}

impl<T: Config> VotingAgent<T> for BifrostAgent<T> {
	fn vtoken(&self) -> CurrencyIdOf<T> {
		self.vtoken
	}

	fn location(&self) -> Location {
		self.location.clone()
	}

	fn delegate_vote(
		&self,
		who: AccountIdOf<T>,
		vtoken: CurrencyIdOf<T>,
		_submitted: bool,
		new_delegator_votes: BTreeMap<PollIndex, VoteItemList<T>>,
		maybe_old_vote: BoundedVec<
			(
				PollIndex,
				OptionalAccountVote<T>,
				Option<ReferendumInfoOf<T>>,
			),
			T::MaxVotes,
		>,
	) -> DispatchResult {
		let call_encode = self.vote_call_encode(new_delegator_votes)?;
		let vote_call: <T as frame_system::Config>::RuntimeCall =
			<T as frame_system::Config>::RuntimeCall::decode(&mut &*call_encode)
				.map_err(|_| Error::<T>::CallDecodeFailed)?;

		// Execute `UtilityCall.as_derivative` via a sovereign address, indirectly voting through the derivative address.
		let sovereign_account =
			polkadot_parachain_primitives::primitives::Sibling::from(T::ParachainId::get())
				.into_account_truncating();
		let origin = RawOrigin::Signed(sovereign_account).into();
		let success = vote_call.dispatch(origin).is_ok();
		Pallet::<T>::handle_vote_result(success, who, vtoken, maybe_old_vote)?;

		if success {
			Ok(())
		} else {
			Err(Error::<T>::InvalidCallDispatch.into())
		}
	}

	fn vote_call_encode(
		&self,
		new_delegator_votes: BTreeMap<PollIndex, VoteItemList<T>>,
	) -> Result<Vec<u8>, Error<T>> {
		let as_derivative = |derivative_index, call| {
			<BifrostCall<T> as UtilityCall<BifrostCall<T>>>::as_derivative(derivative_index, call)
		};

		let mut vote_calls: Vec<(DerivativeIndex, BifrostCall<T>)> = Vec::new();
		for (poll_index, votes) in new_delegator_votes {
			for (derivative_index, vote) in votes {
				let call = <BifrostCall<T> as ConvictionVotingCall<T>>::vote(poll_index, vote);
				vote_calls.push((derivative_index, call));
			}
		}

		// Process based on the number of voting calls:
		// - If there is no voting call, the error `NoData` is returned.
		// - If there is only one voting call, convert it to a call to the derived account and encode the return.
		// - If there are multiple voting calls, convert each call into a call to a derived account, batch these calls together, and encode them before returning.
		match vote_calls.len() {
			0 => Err(Error::<T>::NoData),
			1 => {
				let (derivative_index, call) =
					vote_calls.into_iter().next().ok_or(Error::<T>::NoData)?;
				Ok(as_derivative(derivative_index, call).encode())
			}
			_ => {
				let calls: Vec<_> = vote_calls
					.into_iter()
					.map(|(derivative_index, call)| as_derivative(derivative_index, call))
					.collect();
				Ok(<BifrostCall<T> as UtilityCall<BifrostCall<T>>>::batch_all(calls).encode())
			}
		}
	}

	fn delegate_remove_delegator_vote(
		&self,
		vtoken: CurrencyIdOf<T>,
		poll_index: PollIndex,
		class: PollClass,
		derivative_index: DerivativeIndex,
	) -> DispatchResult {
		let call_encode =
			self.remove_delegator_vote_call_encode(class, poll_index, derivative_index)?;
		let call = <T as frame_system::Config>::RuntimeCall::decode(&mut &*call_encode)
			.map_err(|_| Error::<T>::CallDecodeFailed)?;

		// Execute `UtilityCall.as_derivative` via a sovereign address, indirectly voting through the derivative address.
		let sovereign_account =
			polkadot_parachain_primitives::primitives::Sibling::from(T::ParachainId::get())
				.into_account_truncating();
		let origin = RawOrigin::Signed(sovereign_account).into();
		let success = call.dispatch(origin).is_ok();

		if success {
			Pallet::<T>::handle_remove_delegator_vote_success(vtoken, poll_index);
			Ok(())
		} else {
			Err(Error::<T>::InvalidCallDispatch.into())
		}
	}

	fn remove_delegator_vote_call_encode(
		&self,
		class: PollClass,
		poll_index: PollIndex,
		derivative_index: DerivativeIndex,
	) -> Result<Vec<u8>, Error<T>> {
		let remove_vote_call =
			<BifrostCall<T> as ConvictionVotingCall<T>>::remove_vote(Some(class), poll_index);
		Ok(
			<BifrostCall<T> as UtilityCall<BifrostCall<T>>>::as_derivative(
				derivative_index,
				remove_vote_call,
			)
			.encode(),
		)
	}

	fn block_number(&self) -> BlockNumberFor<T> {
		T::LocalBlockNumberProvider::current_block_number()
	}
}
