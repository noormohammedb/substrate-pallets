#![cfg_attr(not(feature = "std"), no_std)]

use polkadot_sdk_frame::prelude::*;

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod business_logic;

#[polkadot_sdk_frame::pallet]
pub mod pallet {

	use super::*;

	use polkadot_sdk_frame::traits::{Saturating, Zero};

	pub type PalletTokenBalance = u128;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		#[pallet::constant(10)]
		type EpochPeriod: Get<BlockNumberFor<Self>>;

		#[pallet::constant(10)]
		type MintAmountPerEpoch: Get<PalletTokenBalance>;

		#[pallet::constant("706F6F6C")]
		type PoolAddress: Get<Self::AccountId>;
	}

	// to calculate % while claiming rewards
	#[pallet::storage]
	pub type GenesisTotalIssue<T: Config> = StorageValue<_, PalletTokenBalance, ValueQuery>;

	// reward distribution is based on the genesis holding %
	#[pallet::storage]
	pub type GenesisHolders<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, PalletTokenBalance>;

	#[pallet::storage]
	pub type TotalIssue<T: Config> = StorageValue<_, PalletTokenBalance>;

	// (balance, last_claimed_block_number)
	#[pallet::storage]
	pub type Holdings<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, (PalletTokenBalance, BlockNumberFor<T>)>;

	/// hook will read on every block, so its better to whitelisting
	#[pallet::storage]
	#[pallet::whitelist_storage]
	pub type LastMintBlock<T: Config> = StorageValue<_, BlockNumberFor<T>, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub initial_token_distribution: Vec<(T::AccountId, PalletTokenBalance)>,
	}

	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				initial_token_distribution: Default::default(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			let total_issue =
				self.initial_token_distribution
					.iter()
					.fold(0, |acc, (who, balance)| {
						GenesisHolders::<T>::insert(who, balance);
						Holdings::<T>::insert(who, (balance, BlockNumberFor::<T>::zero()));

						acc.saturating_add(*balance)
					});

			TotalIssue::<T>::put(total_issue);
			GenesisTotalIssue::<T>::put(total_issue);
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(current_block: BlockNumberFor<T>) -> Weight {
			let current_epoch = Self::block_to_epoch(current_block);
			if current_epoch > LastMintBlock::<T>::get() {
				let mint_amount = T::MintAmountPerEpoch::get();
				TotalIssue::<T>::mutate(|current_total| {
					if let Some(total_current) = current_total {
						*total_current = total_current.saturating_add(mint_amount);
					}
				});

				Holdings::<T>::mutate(T::PoolAddress::get(), |holding| {
					if let Some((balance_pool, _)) = holding {
						*balance_pool = balance_pool.saturating_sub(mint_amount);
					} else {
						*holding = Some((mint_amount, BlockNumberFor::<T>::zero()));
					}
				});

				LastMintBlock::<T>::set(current_epoch);

				Self::deposit_event(Event::<T>::Mint(mint_amount));

				return T::DbWeight::get().reads_writes(3, 3);
			}

			T::DbWeight::get().reads(1)
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T> {
		Mint(PalletTokenBalance),
		Claim(PalletTokenBalance),
	}

	#[pallet::error]
	pub enum Error<T> {
		GenesisHoldersOnlyEligibleToClaim,
		NoTokenHolding,
		WaitUntilNextEpoch,
		CantClaimZeroToken,
		PoolNotInitialized,
		SomethingWentWrongOnClaimCalculation,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::from_parts(10_000, 10_000))]
		pub fn claim_pool_reward(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::claim_poo_reward_internal(who)?;
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(Weight::from_parts(10_000, 10_000))]
		pub fn example_extrinsic(origin: OriginFor<T>) -> DispatchResult {
			let _who = ensure_signed(origin)?;
			Ok(())
		}
	}
}
