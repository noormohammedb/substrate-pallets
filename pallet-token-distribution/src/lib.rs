//! A shell pallet built with [`frame`].
//!
//! To get started with this pallet, try implementing the guide in
//! <https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/guides/your_first_pallet/index.html>

#![cfg_attr(not(feature = "std"), no_std)]

use polkadot_sdk_frame::prelude::*;

// Re-export all pallet parts, this is needed to properly import the pallet into the runtime.
pub use pallet::*;

#[polkadot_sdk_frame::pallet]
pub mod pallet {
	use super::*;

	// use polkadot_sdk_frame::deps::frame_support::pallet_prelude::*;
	use polkadot_sdk_frame::deps::frame_support::traits::fungible;

	pub type BalanceOf<T> = <<T as Config>::NativeBalance as fungible::Inspect<
		<T as frame_system::Config>::AccountId,
	>>::Balance;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type NativeBalance: fungible::Inspect<Self::AccountId>;
	}

	// to calculate % while claiming rewards
	#[pallet::storage]
	pub type GenesisTotalIssue<T: Config> = StorageValue<_, u32, ValueQuery>;

	// reward distribution is based on the genesis holding %
	#[pallet::storage]
	pub type GenesisHolders<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>, ValueQuery>;

	// (balance, last_claimed_block_number)
	#[pallet::storage]
	pub type Holdings<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, (BalanceOf<T>, BlockNumberFor<T>)>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::from_parts(10_000, 10_000))]
		pub fn example_extrinsic(origin: OriginFor<T>) -> DispatchResult {
			let _who = ensure_signed(origin)?;
			Ok(())
		}
	}
}
