pub trait InspectEvmAccounts<AccountId, EvmAddress> {
	/// get the EVM address from the substrate address.
	fn evm_address(account_id: &impl AsRef<[u8; 32]>) -> EvmAddress;

	/// Get the AccountId from the EVM address.
	fn convert_account_id(evm_address: EvmAddress) -> AccountId;

	/// Return the Substrate address bound to the EVM account. If not bound, returns `None`.
	fn bound_account_id(evm_address: EvmAddress) -> Option<AccountId>;

	/// Get the Substrate address from the EVM address.
	/// Returns the converted address if the address wasn't bind.
	fn account_id(evm_address: EvmAddress) -> AccountId;

	/// Returns `True` if the address is allowed to deploy smart contracts.
	fn can_deploy_contracts(evm_address: EvmAddress) -> bool;
}

impl<AccountId: From<[u8; 32]>, EvmAddress: Default> InspectEvmAccounts<AccountId, EvmAddress>
	for ()
{
	fn evm_address(_account_id: &impl AsRef<[u8; 32]>) -> EvmAddress {
		EvmAddress::default()
	}

	fn convert_account_id(_evm_address: EvmAddress) -> AccountId {
		[0u8; 32].into()
	}

	fn bound_account_id(_evm_address: EvmAddress) -> Option<AccountId> {
		None
	}

	fn account_id(_evm_address: EvmAddress) -> AccountId {
		[0u8; 32].into()
	}

	fn can_deploy_contracts(_evm_address: EvmAddress) -> bool {
		false
	}
}
