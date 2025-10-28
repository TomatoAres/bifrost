use crate::Config;
use frame_support::{pallet_prelude::*, traits::fungible, Deserialize, Serialize};
use parity_scale_codec::DecodeWithMemTracking;

pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub type BalanceOf<T> = <<T as Config>::Fungible as fungible::Inspect<AccountIdOf<T>>>::Balance;

#[derive(
	Debug,
	Default,
	Serialize,
	Deserialize,
	Encode,
	Decode,
	DecodeWithMemTracking,
	Copy,
	Clone,
	PartialEq,
	Eq,
	PartialOrd,
	Ord,
	TypeInfo,
)]
pub struct BridgeConfig {
	pub send_enabled: bool,
	pub receive_enabled: bool,
}

#[derive(
	Debug,
	Default,
	Serialize,
	Deserialize,
	Encode,
	Decode,
	DecodeWithMemTracking,
	Copy,
	Clone,
	PartialEq,
	Eq,
	PartialOrd,
	Ord,
	TypeInfo,
)]
/// XCM fees to be paid at the respective hops. Which is either:
/// 1. AHK -> AHP -> BP
/// 2. AHP -> AHK -> BK
pub struct XcmFeeParams<Balance> {
	/// fees to be paid by sovereign account for source side Asset Hub execution involving swapping to KSM/DOT [BNC]
	#[codec(compact)]
	pub hop1: Balance,
	/// fees to be paid by sovereign account for destination side Asset Hub execution involving swapping to DOT/KSM [KSM/DOT]
	#[codec(compact)]
	pub hop2: Balance,
	/// fees to be paid by sovereign account for destination side Bifrost execution involving swapping to BNC [DOT/KSM]
	#[codec(compact)]
	pub hop3: Balance,
}

pub trait PKBridgeTransferTokens {
	type AccountId;

	type Balance;
	type Location;

	type Error: core::fmt::Debug;

	fn transfer_tokens(
		who: Self::AccountId,
		amount: Self::Balance,
		forward_tokens_to: Option<Self::Location>,
		nonce: u64,
	) -> Result<(), Self::Error>;
}
