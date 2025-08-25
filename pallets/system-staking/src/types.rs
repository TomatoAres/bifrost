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
use crate::RoundIndex;
use frame_support::pallet_prelude::*;
use parity_scale_codec::{Decode, Encode};
use sp_runtime::traits::Zero;
use sp_std::prelude::*;

#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
/// The current round index and transition information
pub struct RoundInfo<BlockNumber> {
	/// Current round index
	pub current: RoundIndex,
	/// The first block of the current round
	pub first: BlockNumber,
	/// The length of the current round in number of blocks
	pub length: u32,
}
impl<
		B: Copy + sp_std::ops::Add<Output = B> + sp_std::ops::Sub<Output = B> + From<u32> + PartialOrd,
	> RoundInfo<B>
{
	pub fn new(current: RoundIndex, first: B, length: u32) -> RoundInfo<B> {
		RoundInfo {
			current,
			first,
			length,
		}
	}
	/// Check if the round should be updated
	pub fn should_update(&self, now: B) -> bool {
		//Current blockNumber -  BlockNumber of Round Start >= Length of Round ==> should update
		now - self.first >= self.length.into()
	}
	/// Update round
	pub fn update(&mut self, now: B) {
		//Set current round index -= 1
		self.current = self.current.saturating_add(1u32);
		//Set blockNumber of Round Start = Current blockNumber
		self.first = now;
	}

	/// Check exec_delay match
	pub fn check_delay(&self, now: B, delay: B) -> bool {
		//Current blockNumber -  BlockNumber of Round Start == delay blockNumber ===> true
		now - self.first == delay && delay != 0.into()
	}
}
impl<
		B: Copy + sp_std::ops::Add<Output = B> + sp_std::ops::Sub<Output = B> + From<u32> + PartialOrd,
	> Default for RoundInfo<B>
{
	fn default() -> RoundInfo<B> {
		RoundInfo::new(1u32, 1u32.into(), 20u32)
	}
}

#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct TokenInfo<
	Balance: Copy,
	BlockNumber: Copy
		+ sp_std::ops::Add<Output = BlockNumber>
		+ sp_std::ops::Sub<Output = BlockNumber>
		+ From<u32>
		+ PartialOrd,
> {
	/// token_config.system_stakable_base
	pub system_stakable_amount: Balance,
	/// Number of additional token already mint
	pub system_shadow_amount: Balance,
	/// Number of pending redemptions
	pub pending_redeem_amount: Balance,
	/// Current TokenConfig
	pub current_config: TokenConfig<Balance, BlockNumber>,
	/// New TokenConfig
	pub new_config: TokenConfig<Balance, BlockNumber>,
}

impl<
		Balance: Zero + Copy,
		BlockNumber: Copy
			+ sp_std::ops::Add<Output = BlockNumber>
			+ sp_std::ops::Sub<Output = BlockNumber>
			+ From<u32>
			+ PartialOrd,
	> Default for TokenInfo<Balance, BlockNumber>
{
	fn default() -> TokenInfo<Balance, BlockNumber> {
		TokenInfo {
			system_stakable_amount: Balance::zero(),
			system_shadow_amount: Balance::zero(),
			pending_redeem_amount: Balance::zero(),
			current_config: TokenConfig::<Balance, BlockNumber>::default(),
			new_config: TokenConfig::<Balance, BlockNumber>::default(),
		}
	}
}

impl<
		Balance: Copy + PartialEq,
		BlockNumber: Copy
			+ sp_std::ops::Add<Output = BlockNumber>
			+ sp_std::ops::Sub<Output = BlockNumber>
			+ From<u32>
			+ PartialOrd,
	> TokenInfo<Balance, BlockNumber>
{
	pub fn check_config_change(&self) -> bool {
		self.current_config != self.new_config
	}

	pub fn update_config(&mut self) {
		self.current_config = self.new_config.clone();
	}
}

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct TokenConfig<Balance, BlockNumber>
where
	BlockNumber: Copy
		+ sp_std::ops::Add<Output = BlockNumber>
		+ sp_std::ops::Sub<Output = BlockNumber>
		+ From<u32>
		+ PartialOrd,
{
	/// Number of blocks with delayed execution
	pub exec_delay: BlockNumber,
	/// System stakable base balance
	pub system_stakable_base: Balance,
}

impl<
		Balance: Zero,
		BlockNumber: Copy
			+ sp_std::ops::Add<Output = BlockNumber>
			+ sp_std::ops::Sub<Output = BlockNumber>
			+ From<u32>
			+ PartialOrd,
	> Default for TokenConfig<Balance, BlockNumber>
{
	fn default() -> TokenConfig<Balance, BlockNumber> {
		TokenConfig {
			exec_delay: 0u32.into(),
			system_stakable_base: Balance::zero(),
		}
	}
}
