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
	xcm_config::{AccountIdToLocation, AssetHubLocation, XcmConfig},
	PKBridge,
};
use alloc::vec;
use bifrost_p_k_bridge::PKBridgeTransferTokens;
use bifrost_primitives::{
	AccountId, AssetHubKusamaGlobalLocation, AssetHubPolkadotGlobalLocation, Balance,
	BifrostKusamaLocation, BifrostPolkadotLocation, RemoteBifrostPolkadotBncLocation,
};
use bifrost_runtime_common::bridge_xcm_helper::send_remote_xcm;
use bifrost_runtime_common::bridge_xcm_helper::{
	bridge_transfer_in_runtime_call, sibling_asset_hub_xcm,
};
use frame_support::ord_parameter_types;
use sp_core::hex2array;
use sp_runtime::traits::Convert;
use xcm::{latest::Location, prelude::XcmError};

ord_parameter_types! {
	pub const BifrostKusamaGlobalSovereignAccount: AccountId = AccountId::new(hex2array!("ef198f9acc4b62413dbf1a7dbe36a085e4525360663c11fcfedeb1305c85da1e"));
}

pub struct TransferTokensToKusama;

impl PKBridgeTransferTokens for TransferTokensToKusama {
	type AccountId = AccountId;
	type Balance = Balance;
	type Location = Location;
	type Error = XcmError;

	fn transfer_tokens(
		who: Self::AccountId,
		amount: Self::Balance,
		location: Option<Self::Location>,
		nonce: u64,
	) -> Result<(), Self::Error> {
		let who_location = AccountIdToLocation::convert(who.clone());
		let fees = PKBridge::xcm_fee_config();

		let remote_xcm = sibling_asset_hub_xcm(
			bridge_transfer_in_runtime_call(who.clone(), amount, location.clone(), nonce),
			(RemoteBifrostPolkadotBncLocation::get(), fees.hop1).into(),
			BifrostPolkadotLocation::get(),
			AssetHubPolkadotGlobalLocation::get(),
			AssetHubKusamaGlobalLocation::get(),
			fees.hop2,
			BifrostKusamaLocation::get(),
			fees.hop3,
		);
		send_remote_xcm::<XcmConfig, <XcmConfig as xcm_executor::Config>::RuntimeCall>(
			who_location,
			AssetHubLocation::get(),
			remote_xcm,
		)?;
		Ok(())
	}
}
