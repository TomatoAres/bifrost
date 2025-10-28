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

//! Common code that the Porter requires to send tokens back and forth between BK<>BP.

use crate::Balance;
use bifrost_p_k_bridge::XcmFeeParams;
use bifrost_primitives::AccountId;
use parity_scale_codec::{Compact, Encode};
use sp_std::vec;
use xcm::latest::Assets;
use xcm::{
	latest::{
		Asset, AssetFilter, AssetTransferFilter::ReserveDeposit, Location, OriginKind, Parent,
		SendXcm, WildAsset, Xcm,
	},
	prelude::{AccountId32, Fungible, XcmError},
};

pub const IK_FEE: u128 = 1000000000000;

// BNC Amount
pub const AHK_FEE: u128 = 30_000_000_000_000;
pub const AHP_FEE: u128 = 3000000000000;
pub const IP_FEE: u128 = 1000000000000;

// hop1: [BNC], hop2: [KSM], hope: [DOT]. Each hop must include the fees of all subsequent hops (consider swapping)
pub const DEFAULT_XCM_FEES_IK_PERSPECTIVE: XcmFeeParams<Balance> = XcmFeeParams {
	// BNC
	hop1: AHK_FEE,
	// AssetHub withdraw fee
	hop2: AHP_FEE,
	hop3: IP_FEE,
};

// hop1: [BNC], hop2: [DOT], hop3: [KSM]. Each hop must include the fees of all subsequent hops (consider swapping)
pub const DEFAULT_XCM_FEES_IP_PERSPECTIVE: XcmFeeParams<Balance> = XcmFeeParams {
	hop1: AHP_FEE,
	hop2: AHK_FEE,
	hop3: IK_FEE,
};

/// The bridge::transfer_in call used by both runtimes.
///
/// We have tests in the runtimes that ensure that the
/// format and the indexes are correct.
pub fn bridge_transfer_in_runtime_call(
	beneficiary: AccountId,
	amount: Balance,
	location: Option<Location>,
	nonce: u64,
) -> (
	[u8; 2],
	AccountId,
	Compact<Balance>,
	Option<Location>,
	Compact<u64>,
) {
	// ([pallet_index, call_index], ...)
	([141, 3], beneficiary, amount.into(), location, nonce.into())
}

#[allow(clippy::too_many_arguments)]
/// XCM to be executed on the first hop, namely the Asset Hub sibling.
/// Bifrost -> AssetHub
/// call: The call to be executed on cousin Bifrost after the two hops.
/// teleported_asset: The asset(BNC) that was teleported to pay for the first hop.
/// sibling_bifrost_location: The location of the sibling Bifrost chain. Refunded assets will be deposited there.
/// cousin_bifrost_location: The location of the cousin Bifrost chain. Refunded assets will be deposited there.
/// cousin_asset_hub_location: Xcm will be sent to this location.
pub fn sibling_asset_hub_xcm<Call, BifrostRuntimeCall: Encode>(
	call: BifrostRuntimeCall,
	teleported_asset: Asset,
	sibling_bifrost_location: Location,
	cousin_bifrost_location: Location,
	cousin_asset_hub_location: Location,
	sibling_asset_hub_fee_amount: Balance,
	sibling_bifrost_location_relative_to_cousin_asset_hub: Location,
	cousin_asset_hub_fee_amount: Balance,
	treasury_account: AccountId,
) -> Xcm<Call> {
	let refund_location = sibling_bifrost_location;
	let destination = cousin_asset_hub_location;
	let fee_assets: Assets = Asset {
		id: Parent.into(),
		fun: Fungible(sibling_asset_hub_fee_amount),
	}
	.into();
	let xcm_for_destination = cousin_asset_hub_xcm(
		call,
		cousin_bifrost_location,
		sibling_bifrost_location_relative_to_cousin_asset_hub,
		cousin_asset_hub_fee_amount,
		treasury_account,
	);
	Xcm::<Call>::builder_unsafe()
		.receive_teleported_asset(teleported_asset.clone())
		.pay_fees(teleported_asset)
		.set_appendix(
			Xcm::<Call>::builder_unsafe()
				.refund_surplus()
				.deposit_asset(AssetFilter::Wild(WildAsset::All), refund_location)
				.build(),
		)
		.withdraw_asset(fee_assets.clone())
		// Send to asset hub global location
		.initiate_transfer(
			destination,
			Some(ReserveDeposit(AssetFilter::Definite(fee_assets))),
			true,
			vec![],
			xcm_for_destination,
		)
		.build()
}

/// Nested XCM to be executed as `remote_xcm` from within `sibling_asset_hub_xcm` on the
/// second hop, namely the Asset Hub cousin.
fn cousin_asset_hub_xcm<Call, BifrostRuntimeCall: Encode>(
	call: BifrostRuntimeCall,
	cousin_bifrost_location: Location,
	sibling_bifrost_location: Location,
	fee_amount: Balance,
	treasury_account: AccountId,
) -> Xcm<Call> {
	// Refund to sovereign account of cousin bifrost on asset hub
	let refund_location = cousin_bifrost_location;
	let destination = sibling_bifrost_location;
	let fee_assets: Assets = Asset {
		id: Parent.into(),
		fun: Fungible(fee_amount),
	}
	.into();
	Xcm::<Call>::builder_unsafe()
		.set_appendix(
			Xcm::<Call>::builder_unsafe()
				.refund_surplus()
				.deposit_asset(AssetFilter::Wild(WildAsset::All), refund_location)
				.build(),
		)
		.withdraw_asset(fee_assets.clone())
		.initiate_transfer(
			destination,
			Some(ReserveDeposit(AssetFilter::Definite(fee_assets))),
			true,
			vec![],
			bifrost_transact_xcm(call, treasury_account),
		)
		.build()
}

fn bifrost_transact_xcm<Call, BifrostRuntimeCall: Encode>(
	call: BifrostRuntimeCall,
	treasury_account: AccountId,
) -> Xcm<Call> {
	Xcm::<Call>::builder_unsafe()
		.set_appendix(
			Xcm::<Call>::builder_unsafe()
				.refund_surplus()
				.deposit_asset(
					AssetFilter::Wild(WildAsset::All),
					AccountId32 {
						network: None,
						id: treasury_account.into(),
					},
				)
				.build(),
		)
		.transact(OriginKind::SovereignAccount, None, call.encode())
		.build()
}

/// Executes a pair of local and remote XCMs, e.g. burn locally and if successful send
/// the XCM to mint assets on the remote chain.
pub fn send_remote_xcm<XcmConfig: xcm_executor::Config<RuntimeCall = Call>, Call>(
	_who: Location,
	destination: Location,
	remote_xcm: Xcm<()>,
) -> Result<(), XcmError> {
	let (ticket, _delivery_fees) = <XcmConfig as xcm_executor::Config>::XcmSender::validate(
		&mut Some(destination),
		&mut Some(remote_xcm),
	)?;

	<XcmConfig as xcm_executor::Config>::XcmSender::deliver(ticket)?;
	Ok(())
}
