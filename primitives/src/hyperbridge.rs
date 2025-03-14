use crate::CurrencyId;
use ismp::host::StateMachine;
use pallet_ismp::ModuleId;
use sp_core::{H160, H256};
use sp_runtime::DispatchError;
use sp_std::vec::Vec;

/// Hyperbridge sender trait
pub trait HyperBridgeSender<AccountId, Balance> {
	/// Send message to the destination chain
	/// Parameters
	/// - `from`: Module id of the sender
	/// - `to`: Destination contract address
	/// - `dest`: Destination state machine
	/// - `msg`: Message to be sent
	/// - `timeout`: Timeout for the message
	/// Returns
	/// - `H256`: Message hash
	fn send_msg(
		from: ModuleId,
		to: H160,
		dest: StateMachine,
		msg: Vec<u8>,
		timeout: u64,
	) -> Result<H256, DispatchError>;

	/// Send asset to the destination chain and call a function
	/// Parameters
	/// - `currency_id`: Currency id of the asset
	/// - `to`: Destination contract address
	/// - `dest`: Destination state machine
	/// - `amount`: Amount to be sent
	/// - `timeout`: Timeout for the message
	/// - `data`: Call data
	/// Returns
	/// - `H256`: Message hash
	fn send_and_call(
		currency_id: CurrencyId,
		from: AccountId,
		to: H160,
		dest: StateMachine,
		amount: Balance,
		timeout: u64,
		data: Option<Vec<u8>>,
	) -> Result<H256, DispatchError>;
}
