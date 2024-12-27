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

use bifrost_primitives::{Rate, Ratio};
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::{DispatchError, FixedU128, RuntimeDebug};
use sp_std::prelude::*;

pub trait LendMarket<CurrencyId, AccountId, Balance> {
	fn do_mint(
		supplier: &AccountId,
		asset_id: CurrencyId,
		amount: Balance,
	) -> Result<(), DispatchError>;
	fn do_borrow(
		borrower: &AccountId,
		asset_id: CurrencyId,
		amount: Balance,
	) -> Result<(), DispatchError>;
	fn do_collateral_asset(
		supplier: &AccountId,
		asset_id: CurrencyId,
		enable: bool,
	) -> Result<(), DispatchError>;
	fn do_repay_borrow(
		borrower: &AccountId,
		asset_id: CurrencyId,
		amount: Balance,
	) -> Result<(), DispatchError>;
	fn do_redeem(
		supplier: &AccountId,
		asset_id: CurrencyId,
		amount: Balance,
	) -> Result<(), DispatchError>;
}

pub trait LendMarketPositionDataProvider<CurrencyId, AccountId, Balance> {
	fn get_current_borrow_balance(
		borrower: &AccountId,
		asset_id: CurrencyId,
	) -> Result<Balance, DispatchError>;

	fn get_current_collateral_balance(
		supplier: &AccountId,
		asset_id: CurrencyId,
	) -> Result<Balance, DispatchError>;
}

pub trait LendMarketMarketDataProvider<CurrencyId, Balance> {
	fn get_market_info(asset_id: CurrencyId) -> Result<MarketInfo, DispatchError>;
	fn get_market_status(asset_id: CurrencyId) -> Result<MarketStatus<Balance>, DispatchError>;
	// for compatibility we keep this func
	fn get_full_interest_rate(asset_id: CurrencyId) -> Option<Rate>;
}

/// MarketInfo contains some static attrs as a subset of Market struct in LendMarket
#[derive(Default, Copy, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct MarketInfo {
	pub collateral_factor: Ratio,
	pub liquidation_threshold: Ratio,
	pub reserve_factor: Ratio,
	pub close_factor: Ratio,
	pub full_rate: Rate,
}

/// MarketStatus contains some dynamic calculated attrs of Market
#[derive(Default, Copy, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct MarketStatus<Balance> {
	pub borrow_rate: Rate,
	pub supply_rate: Rate,
	pub exchange_rate: Rate,
	pub utilization: Ratio,
	pub total_borrows: Balance,
	pub total_reserves: Balance,
	pub borrow_index: FixedU128,
}
