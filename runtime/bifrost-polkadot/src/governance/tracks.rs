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

//! Track configurations for governance.

use super::*;
const fn percent(x: i32) -> sp_runtime::FixedI64 {
	sp_runtime::FixedI64::from_rational(x as u128, 100)
}
const fn permill(x: i32) -> sp_runtime::FixedI64 {
	sp_runtime::FixedI64::from_rational(x as u128, 1000)
}

use pallet_referenda::Curve;
use sp_runtime::str_array;

const TRACKS_DATA: [pallet_referenda::Track<u16, Balance, BlockNumber>; 9] = [
	pallet_referenda::Track {
		id: 0,
		info: pallet_referenda::TrackInfo {
			// Name of this track.
			name: str_array("root"),
			// A limit for the number of referenda on this track that can be being decided at once.
			// For Root origin this should generally be just one.
			max_deciding: 1,
			// Amount that must be placed on deposit before a decision can be made.
			decision_deposit: 50_000 * BNCS,
			// Amount of time this must be submitted for before a decision can be made.
			prepare_period: 2 * HOURS,
			// Amount of time that a decision may take to be approved prior to cancellation.
			decision_period: 14 * DAYS,
			// Amount of time that the approval criteria must hold before it can be approved.
			confirm_period: DAYS,
			// Minimum amount of time that an approved proposal must be in the dispatch queue.
			min_enactment_period: DAYS,
			// Minimum aye votes as percentage of overall conviction-weighted votes needed for
			// approval as a function of time into decision period.
			min_approval: Curve::make_reciprocal(4, 28, percent(80), percent(50), percent(100)),
			// Minimum pre-conviction aye-votes ("support") as percentage of overall population that
			// is needed for approval as a function of time into decision period.
			min_support: Curve::make_linear(28, 28, permill(0), percent(50)),
		},
	},
	pallet_referenda::Track {
		id: 1,
		info: pallet_referenda::TrackInfo {
			name: str_array("whitelisted_caller"),
			max_deciding: 100,
			decision_deposit: 5_000 * BNCS,
			prepare_period: 5 * MINUTES,
			decision_period: 14 * DAYS,
			confirm_period: 5 * MINUTES,
			min_enactment_period: 5 * MINUTES,
			min_approval: Curve::make_reciprocal(
				16,
				28 * 24,
				percent(96),
				percent(50),
				percent(100),
			),
			min_support: Curve::make_reciprocal(1, 1792, percent(3), percent(2), percent(50)),
		},
	},
	pallet_referenda::Track {
		id: 2,
		info: pallet_referenda::TrackInfo {
			name: str_array("fellowship_admin"),
			max_deciding: 10,
			decision_deposit: 2_500 * BNCS,
			prepare_period: 2 * HOURS,
			decision_period: 14 * DAYS,
			confirm_period: 3 * HOURS,
			min_enactment_period: 10 * MINUTES,
			min_approval: Curve::make_linear(17, 28, percent(50), percent(100)),
			min_support: Curve::make_reciprocal(12, 28, percent(1), percent(0), percent(50)),
		},
	},
	pallet_referenda::Track {
		id: 3,
		info: pallet_referenda::TrackInfo {
			name: str_array("referendum_canceller"),
			max_deciding: 1_000,
			decision_deposit: 5_000 * BNCS,
			prepare_period: 2 * HOURS,
			decision_period: 7 * DAYS,
			confirm_period: 3 * HOURS,
			min_enactment_period: 10 * MINUTES,
			min_approval: Curve::make_linear(17, 28, percent(50), percent(100)),
			min_support: Curve::make_reciprocal(12, 28, percent(1), percent(0), percent(50)),
		},
	},
	pallet_referenda::Track {
		id: 4,
		info: pallet_referenda::TrackInfo {
			name: str_array("referendum_killer"),
			max_deciding: 1_000,
			decision_deposit: 25_000 * BNCS,
			prepare_period: 2 * HOURS,
			decision_period: 14 * DAYS,
			confirm_period: 3 * HOURS,
			min_enactment_period: 10 * MINUTES,
			min_approval: Curve::make_linear(17, 28, percent(50), percent(100)),
			min_support: Curve::make_reciprocal(12, 28, percent(1), percent(0), percent(50)),
		},
	},
	pallet_referenda::Track {
		id: 10,
		info: pallet_referenda::TrackInfo {
			name: str_array("liquid_staking"),
			max_deciding: 10,
			decision_deposit: 2_500 * BNCS,
			prepare_period: 2 * HOURS,
			decision_period: 14 * DAYS,
			confirm_period: 3 * HOURS,
			min_enactment_period: 10 * MINUTES,
			min_approval: Curve::make_reciprocal(2, 28, percent(80), percent(50), percent(100)),
			min_support: Curve::make_reciprocal(2, 28, percent(5), percent(0), percent(50)),
		},
	},
	pallet_referenda::Track {
		id: 12,
		info: pallet_referenda::TrackInfo {
			name: str_array("salp_admin"),
			max_deciding: 10,
			decision_deposit: 2_500 * BNCS,
			prepare_period: 15 * MINUTES,
			decision_period: 14 * DAYS,
			confirm_period: HOURS,
			min_enactment_period: 10 * MINUTES,
			min_approval: Curve::make_reciprocal(2, 28, percent(80), percent(50), percent(100)),
			min_support: Curve::make_reciprocal(2, 28, percent(5), percent(0), percent(50)),
		},
	},
	pallet_referenda::Track {
		id: 13,
		info: pallet_referenda::TrackInfo {
			name: str_array("treasury_spend"),
			max_deciding: 100,
			decision_deposit: 500 * BNCS,
			prepare_period: 2 * HOURS,
			decision_period: 14 * DAYS,
			confirm_period: 3 * HOURS,
			min_enactment_period: 10 * MINUTES,
			min_approval: Curve::make_linear(23, 28, percent(50), percent(100)),
			min_support: Curve::make_reciprocal(16, 28, percent(1), percent(0), percent(50)),
		},
	},
	pallet_referenda::Track {
		id: 14,
		info: pallet_referenda::TrackInfo {
			name: str_array("delegated_voting_admin"),
			max_deciding: 100,
			decision_deposit: 2_500 * BNCS,
			prepare_period: 2 * HOURS,
			decision_period: 7 * DAYS,
			confirm_period: DAYS,
			min_enactment_period: DAYS,
			// Minimum aye votes as percentage of overall conviction-weighted votes needed for
			// approval as a function of time into decision period.
			min_approval: Curve::make_reciprocal(4, 28, percent(80), percent(50), percent(100)),
			// Minimum pre-conviction aye-votes ("support") as percentage of overall population that
			// is needed for approval as a function of time into decision period.
			min_support: Curve::make_linear(28, 28, permill(0), percent(50)),
		},
	},
];

pub struct TracksInfo;
impl pallet_referenda::TracksInfo<Balance, BlockNumber> for TracksInfo {
	type Id = u16;
	type RuntimeOrigin = <RuntimeOrigin as frame_support::traits::OriginTrait>::PalletsOrigin;
	fn tracks(
	) -> impl Iterator<Item = Cow<'static, pallet_referenda::Track<Self::Id, Balance, BlockNumber>>>
	{
		TRACKS_DATA.iter().map(Cow::Borrowed)
	}
	fn track_for(id: &Self::RuntimeOrigin) -> Result<Self::Id, ()> {
		if let Ok(system_origin) = frame_system::RawOrigin::try_from(id.clone()) {
			match system_origin {
				frame_system::RawOrigin::Root => Ok(0),
				_ => Err(()),
			}
		} else if let Ok(custom_origin) = origins::Origin::try_from(id.clone()) {
			match custom_origin {
				origins::Origin::WhitelistedCaller => Ok(1),
				origins::Origin::FellowshipAdmin => Ok(2),
				origins::Origin::ReferendumCanceller => Ok(3),
				origins::Origin::ReferendumKiller => Ok(4),
				origins::Origin::LiquidStaking => Ok(10),
				origins::Origin::SALPAdmin => Ok(12),
				origins::Origin::TreasurySpend => Ok(13),
				origins::Origin::DelegatedVotingAdmin => Ok(14),
				_ => Err(()),
			}
		} else {
			Err(())
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use scale_info::prelude::string::String;
	use sp_std::str::FromStr;

	#[test]
	/// To ensure voters are always locked into their vote
	fn vote_locking_always_longer_than_enactment_period() {
		for track in TRACKS_DATA {
			assert!(
				<Runtime as pallet_conviction_voting::Config>::VoteLockingPeriod::get()
					>= track.info.min_enactment_period,
				"Track {} has enactment period {} < vote locking period {}",
				from_str_array(&track.info.name),
				track.info.min_enactment_period,
				<Runtime as pallet_conviction_voting::Config>::VoteLockingPeriod::get(),
			);
		}
	}

	#[test]
	fn all_tracks_have_origins() {
		for track in TRACKS_DATA {
			// check name.into() is successful either converts into "root" or custom origin
			let track_is_root = track.info.name == str_array("root");
			let track_has_custom_origin =
				custom_origins::Origin::from_str(&from_str_array(&track.info.name)).is_ok();
			assert!(track_is_root || track_has_custom_origin);
		}
	}

	fn from_str_array<const N: usize>(arr: &[u8; N]) -> String {
		let len = arr.iter().position(|&c| c == 0).unwrap_or(N);
		String::from_utf8_lossy(&arr[..len]).into_owned()
	}
}
