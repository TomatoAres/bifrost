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
use frame_support::traits::OnRuntimeUpgrade;
use hex_literal::hex;
#[cfg(feature = "try-runtime")]
use sp_runtime::TryRuntimeError;

const LOG_TARGET: &str = "SLP::migration";

pub struct SlpMigration<T>(sp_std::marker::PhantomData<T>);
impl<T: Config> OnRuntimeUpgrade for SlpMigration<T> {
	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		// Check the storage version
		let in_code_version = Pallet::<T>::in_code_storage_version();
		let on_chain_version = Pallet::<T>::on_chain_storage_version();

		if on_chain_version == 3 && in_code_version == 4 {
			let bnc_0: [u8; 32] =
				hex!["a471c7aa909cc665212bb36003c52c5d3eeec39f96556a8242e861c5dd7dde41"];
			let bnc_0_bn: BlockNumberFor<T> = BlockNumberFor::<T>::from(9255541u32);
			let bnc_1: [u8; 32] =
				hex!["5ecc1d4e60a92262c1bec62d034c979f42cbd3fb1c28570d5baed6e5ed20d533"];
			let bnc_1_bn: BlockNumberFor<T> = BlockNumberFor::<T>::from(9255541u32);
			let bnc_2: [u8; 32] =
				hex!["c83b0bba37f25f365e26efbe6c9ecfa7905dbdc0b0e3ae60b29980b42c509c6f"];
			let bnc_2_bn: BlockNumberFor<T> = BlockNumberFor::<T>::from(9255541u32);

			let dot_0: [u8; 32] =
				hex!("b899b7505a1044ec1ec0495d0a0a799b13196e9b4123daccdae54f69dbfe4a4b");
			let dot_0_bn: BlockNumberFor<T> = BlockNumberFor::<T>::from(8041445u32);
			let dot_1: [u8; 32] =
				hex!("cc40832c2a830dcf1ff86b800671aa813b208df6786b10f6d668905a8f21f17d");
			let dot_1_bn: BlockNumberFor<T> = BlockNumberFor::<T>::from(9255541u32);
			let dot_2: [u8; 32] =
				hex!("0aa18c1ff67dccbb98cbb86ea9808b63dc72582c602fb0dcd7e6716dd9ed9c75");
			let dot_2_bn: BlockNumberFor<T> = BlockNumberFor::<T>::from(9336771u32);

			let glmr: [u8; 20] = hex!("ca98d4378393040408100f490bf98b03f5e7deb7");
			let glmr_bn: BlockNumberFor<T> = BlockNumberFor::<T>::from(9255541u32);

			let ksm_0: [u8; 32] =
				hex!("a43b2797bd4dd454d7fb0870a2a4edd62b39eea0801f6baaf09b05c8634b5a25");
			let ksm_0_bn: BlockNumberFor<T> = BlockNumberFor::<T>::from(9946014u32);
			let ksm_1: [u8; 32] =
				hex!("384e257ac2372c996a4180f6d9a9a0e16631cc76929c600468583e8d798c1760");
			let ksm_1_bn: BlockNumberFor<T> = BlockNumberFor::<T>::from(9946014u32);
			let ksm_2: [u8; 32] =
				hex!("fc68a0ac75740597083ed106a906be06ea18ee1a8afa8c515a45f0919348142c");
			let ksm_2_bn: BlockNumberFor<T> = BlockNumberFor::<T>::from(11099931u32);
			let ksm_3: [u8; 32] =
				hex!("d08c1dda79bf0dd33773344372601cd6a1ff692d8cc365e986d08ca31528b403");
			let ksm_3_bn: BlockNumberFor<T> = BlockNumberFor::<T>::from(11099931u32);
			let ksm_4: [u8; 32] =
				hex!("a9dfa85f2d0b1604104afd100b303fd284dd22fb0f41bb8c535e83b5bd3b4745");
			let ksm_4_bn: BlockNumberFor<T> = BlockNumberFor::<T>::from(11108872u32);
			let ksm_5: [u8; 32] =
				hex!("e2ed4f7c4222c9ee8a30640f2c141ad456bfdf2dba5334b8fee4b463cb137770");
			let ksm_5_bn: BlockNumberFor<T> = BlockNumberFor::<T>::from(11108872u32);

			let movr_0: [u8; 20] = hex!("e8ebdefcdf683425b0147ce811d89c977a8d2fa9");
			let movr_0_bn: BlockNumberFor<T> = BlockNumberFor::<T>::from(11099931u32);
			let movr_1: [u8; 20] = hex!("497478575ff499fa1b2ca8ce5a01c1e676408829");
			let movr_1_bn: BlockNumberFor<T> = BlockNumberFor::<T>::from(11099931u32);

			// Transform storage values
			// We transform the storage values from the old into the new format.
			log::info!(target: LOG_TARGET, "Start to migrate ValidatorBoostList storage...");
			//migrate the value type of ValidatorBoostList
			ValidatorBoostList::<T>::translate(
				|k: CurrencyId,
				 old_list: BoundedVec<(MultiLocation, BlockNumberFor<T>), T::MaxLengthLimit>| {
					log::info!(target: LOG_TARGET, "Migrated to boundedvec for {k:?}...");

					match k {
						k if k == BNC => {
							let mapping = vec![
								(bnc_0.to_vec(), bnc_0_bn),
								(bnc_1.to_vec(), bnc_1_bn),
								(bnc_2.to_vec(), bnc_2_bn),
							];
							Some(map_location_block::<T, _, [u8; 32]>(
								old_list,
								Pallet::<T>::multilocation_to_account_32,
								&mapping,
							))
						}
						k if k == DOT => {
							let mapping = vec![
								(dot_0.to_vec(), dot_0_bn),
								(dot_1.to_vec(), dot_1_bn),
								(dot_2.to_vec(), dot_2_bn),
							];
							Some(map_location_block::<T, _, [u8; 32]>(
								old_list,
								Pallet::<T>::multilocation_to_account_32,
								&mapping,
							))
						}
						k if k == GLMR => {
							let mapping = vec![(glmr.to_vec(), glmr_bn)];
							Some(map_location_block::<T, _, [u8; 20]>(
								old_list,
								Pallet::<T>::multilocation_to_account_20,
								&mapping,
							))
						}
						k if k == KSM => {
							let mapping = vec![
								(ksm_0.to_vec(), ksm_0_bn),
								(ksm_1.to_vec(), ksm_1_bn),
								(ksm_2.to_vec(), ksm_2_bn),
								(ksm_3.to_vec(), ksm_3_bn),
								(ksm_4.to_vec(), ksm_4_bn),
								(ksm_5.to_vec(), ksm_5_bn),
							];
							Some(map_location_block::<T, _, [u8; 32]>(
								old_list,
								Pallet::<T>::multilocation_to_account_32,
								&mapping,
							))
						}
						k if k == MOVR => {
							let mapping =
								vec![(movr_0.to_vec(), movr_0_bn), (movr_1.to_vec(), movr_1_bn)];
							Some(map_location_block::<T, _, [u8; 20]>(
								old_list,
								Pallet::<T>::multilocation_to_account_20,
								&mapping,
							))
						}
						_ => Some(old_list),
					}
				},
			);

			// Update the storage version
			StorageVersion::new(4).put::<Pallet<T>>();

			// Return the consumed weight
			let count = ValidatorBoostList::<T>::iter().count();
			T::DbWeight::get().reads_writes(count as u64 + 1, count as u64 + 1)
		} else {
			// We don't do anything here.
			Weight::zero()
		}
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
		let validator_boost_list_cnt = ValidatorBoostList::<T>::iter().count();
		log::info!(
			target: LOG_TARGET,
			"ValidatorBoostList pre-migrate storage count: {validator_boost_list_cnt:?}",
		);

		let cnt = validator_boost_list_cnt as u32;
		Ok(cnt.encode())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(cnt: Vec<u8>) -> Result<(), TryRuntimeError> {
		let validator_boost_list_cnt_old: u32 = Decode::decode(&mut cnt.as_slice())
			.expect("the state parameter should be something that was generated by pre_upgrade");

		let validator_boost_list_cnt_new = ValidatorBoostList::<T>::iter().count();
		log::info!(
			target: LOG_TARGET,
			"ValidatorBoostList post-migrate storage count: {:?}",
			ValidatorBoostList::<T>::iter().count()
		);
		ensure!(
			validator_boost_list_cnt_new as u32 == validator_boost_list_cnt_old,
			"ValidatorBoostList post-migrate storage count not match"
		);

		Ok(())
	}
}

fn map_location_block<T, F, A>(
	list: BoundedVec<(MultiLocation, BlockNumberFor<T>), T::MaxLengthLimit>,
	convert: F,
	mapping: &[(Vec<u8>, BlockNumberFor<T>)],
) -> BoundedVec<(MultiLocation, BlockNumberFor<T>), T::MaxLengthLimit>
where
	T: Config,
	F: Fn(&MultiLocation) -> Result<A, pallet::Error<T>>,
	A: AsRef<[u8]>,
{
	list.into_iter()
		.map(|(loc, block)| {
			if let Ok(account) = convert(&loc) {
				for (target, new_block) in mapping {
					if account.as_ref() == target.as_slice() {
						return (loc, *new_block);
					}
				}
			}
			(loc, block)
		})
		.collect::<Vec<_>>()
		.try_into()
		.expect("New BoundedVec should not exceed MaxLengthLimit")
}
