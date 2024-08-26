//! A shell pallet built with [`frame`].
//!
//! To get started with this pallet, try implementing the guide in
//! <https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/guides/your_first_pallet/index.html>

#![cfg_attr(not(feature = "std"), no_std)]

use polkadot_sdk_frame::prelude::*;

// Re-export all pallet parts, this is needed to properly import the pallet into the runtime.
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[polkadot_sdk_frame::pallet]
pub mod pallet {

	use super::*;

	use polkadot_sdk_frame::traits::{CheckedDiv, SaturatedConversion, Saturating, Zero};

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
	pub type LastMintBlock<T: Config> = StorageValue<_, BlockNumberFor<T>>;

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
			if current_epoch > LastMintBlock::<T>::get().unwrap_or_default() {
				// mint new token to pool
				// update last minted block
				// update total issue

				let mint_amount = T::MintAmountPerEpoch::get();
				TotalIssue::<T>::mutate_extant(|current_total| {
					current_total.saturating_add(mint_amount)
				});

				Holdings::<T>::mutate_extant(T::PoolAddress::get(), |(pool_balance, _)| {
					(pool_balance.saturating_sub(mint_amount), current_block)
				});
				LastMintBlock::<T>::set(Some(current_epoch));

				return T::DbWeight::get().reads_writes(3, 3);
			}

			T::DbWeight::get().reads(1)
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T> {
		SomeEvent,
		Claim(PalletTokenBalance),
	}

	#[pallet::error]
	pub enum Error<T> {
		SomeError,
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
			let genesis_holding_data = GenesisHolders::<T>::get(&who);
			ensure!(
				genesis_holding_data.is_some(),
				Error::<T>::GenesisHoldersOnlyEligibleToClaim
			);

			let current_epoch = Self::current_epoch();

			let holding_data = Holdings::<T>::get(&who);
			ensure!(holding_data.is_some(), Error::<T>::NoTokenHolding);

			let (_token_balance, last_claimed_block) = holding_data.unwrap_or_default();
			let last_claimed_epoch = Self::block_to_epoch(last_claimed_block); // impl epoch calculation logic later

			ensure!(
				current_epoch > last_claimed_epoch,
				Error::<T>::WaitUntilNextEpoch
			);

			// calculating how many epoch elapsed since last claim
			// then calculate reward based on the % of holding at genesis
			// update both token balance and last_claimed_block_number

			let elapsed_epochs: u128 = ((current_epoch.saturating_sub(last_claimed_epoch))
				.checked_div(&T::EpochPeriod::get()))
			.unwrap_or_default()
			.saturated_into();

			let total_mited_to_pool_from_last_claim =
				T::MintAmountPerEpoch::get().saturating_mul(elapsed_epochs);

			let claimable_token = genesis_holding_data
				.unwrap_or_default()
				.saturating_mul(total_mited_to_pool_from_last_claim)
				.saturating_div(GenesisTotalIssue::<T>::get());

			// some sanity checks
			ensure!(claimable_token > 0, Error::<T>::CantClaimZeroToken);
			let pool_holding_data = Holdings::<T>::get(&T::PoolAddress::get());
			ensure!(pool_holding_data.is_some(), Error::<T>::PoolNotInitialized);

			let pool_balance = pool_holding_data.unwrap_or_default().0;
			ensure!(
				pool_balance >= claimable_token,
				Error::<T>::SomethingWentWrongOnClaimCalculation
			);

			Self::claim_transfer(who, claimable_token);

			Self::deposit_event(Event::<T>::Claim(claimable_token));

			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(Weight::from_parts(10_000, 10_000))]
		pub fn example_extrinsic(origin: OriginFor<T>) -> DispatchResult {
			let _who = ensure_signed(origin)?;
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		// utility functions
		pub fn current_epoch() -> BlockNumberFor<T> {
			Self::block_to_epoch(<frame_system::Pallet<T>>::block_number())
		}

		pub fn block_to_epoch(block_number: BlockNumberFor<T>) -> BlockNumberFor<T> {
			let epoch_period = T::EpochPeriod::get();
			(block_number / epoch_period) * epoch_period
		}

		pub fn claim_transfer(to: T::AccountId, amount: PalletTokenBalance) {
			let current_block = <frame_system::Pallet<T>>::block_number();

			Holdings::<T>::mutate_extant(
				T::PoolAddress::get(),
				|(pool_balance, last_minted_block)| {
					(pool_balance.saturating_sub(amount), *last_minted_block)
				},
			);

			Holdings::<T>::mutate_extant(to, |(user_balance, _)| {
				(user_balance.saturating_add(amount), current_block)
			});
		}
	}
}
