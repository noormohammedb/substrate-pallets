use super::*;
use polkadot_sdk_frame::traits::{CheckedDiv, SaturatedConversion, Saturating};

impl<T: Config> Pallet<T> {
	pub fn claim_poo_reward_internal(who: T::AccountId) -> DispatchResult {
		let genesis_holding_data = GenesisHolders::<T>::get(&who);
		ensure!(
			genesis_holding_data.is_some(),
			Error::<T>::GenesisHoldersOnlyEligibleToClaim
		);

		let current_epoch = Self::current_epoch();

		let holding_data = Holdings::<T>::get(&who);
		ensure!(holding_data.is_some(), Error::<T>::NoTokenHolding);

		let (_token_balance, last_claimed_block) = holding_data.unwrap_or_default();
		let last_claimed_epoch = Self::block_to_epoch(last_claimed_block);

		ensure!(
			current_epoch > last_claimed_epoch,
			Error::<T>::WaitUntilNextEpoch
		);

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
}

// utility functions
impl<T: Config> Pallet<T> {
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
