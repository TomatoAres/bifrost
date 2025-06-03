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

use bifrost_primitives::Balance;
use frame_support::pallet_prelude::{Decode, Encode, MaxEncodedLen, TypeInfo};
use sp_runtime::Saturating;

/// Dapp staking extrinsic call.
#[derive(Encode, Decode, MaxEncodedLen, Clone, Copy, Debug, PartialEq, Eq, TypeInfo)]
pub enum EthereumStaking {
	Stake(#[codec(compact)] Balance),
	Unstake(#[codec(compact)] Balance),
}

/// Ethereum staking ledger.
#[derive(Encode, Decode, MaxEncodedLen, Clone, Debug, Default, PartialEq, Eq, TypeInfo)]
pub struct EthereumStakingLedger {
	/// How much active locked amount an account has. This can be used for staking.
	#[codec(compact)]
	pub locked: Balance,
}

impl EthereumStakingLedger {
	/// Adds the specified amount to the total locked amount.
	pub fn add_lock_amount(&mut self, amount: Balance) {
		self.locked.saturating_accrue(amount);
	}

	/// Subtracts the specified amount of the total locked amount.
	pub fn subtract_lock_amount(&mut self, amount: Balance) {
		self.locked.saturating_reduce(amount);
	}
}
