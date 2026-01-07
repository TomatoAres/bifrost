use crate::*;
use sp_core::H160;
use sp_runtime::DispatchError;
use sp_std::vec::Vec;

#[derive(
	Clone, Debug, PartialEq, Encode, Decode, TypeInfo, MaxEncodedLen, DecodeWithMemTracking,
)]
pub enum BridgeType<AccountId> {
	Parachain(u32, AccountId),
	ParachainEvm(u32, H160),
	HyperBridge(u32, H160),
	SnowBridge(H160),
}

#[derive(
	Clone, Debug, PartialEq, Encode, Decode, TypeInfo, MaxEncodedLen, DecodeWithMemTracking,
)]
pub struct BridgeAsset<Balance> {
	pub currency_id: CurrencyId,
	pub amount: Balance,
}

impl<Balance> From<(CurrencyId, Balance)> for BridgeAsset<Balance> {
	fn from((currency_id, amount): (CurrencyId, Balance)) -> Self {
		BridgeAsset {
			currency_id,
			amount,
		}
	}
}

/// A trait for sending assets across chains
pub trait XChainSender<AccountId, Balance> {
	/// Transfer assets from one account to another across chains
	/// - `from`: The account from which the assets are sent
	/// - `to`: The destination bridge type and account
	/// - `assets`: A vector of assets to be transferred
	/// - `fee_asset_item`: The index of the asset in the `assets` vector to be used for paying fees
	fn do_transfer_assets(
		from: AccountId,
		to: BridgeType<AccountId>,
		assets: Vec<BridgeAsset<Balance>>,
		fee_asset_item: u32,
	) -> Result<(), DispatchError>;
}

impl<AccountId, Balance> XChainSender<AccountId, Balance> for () {
	fn do_transfer_assets(
		_from: AccountId,
		_to: BridgeType<AccountId>,
		_assets: Vec<BridgeAsset<Balance>>,
		_fee_asset_item: u32,
	) -> Result<(), DispatchError> {
		Ok(())
	}
}
