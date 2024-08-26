use crate::{mock::*, *};
use polkadot_sdk_frame::testing_prelude::*;
use polkadot_sdk_frame::traits::OnInitialize;

#[test]
fn genesis_total_issue() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let configured_genesis = genesis_data();

		let total_issue = configured_genesis.iter().fold(0, |acc, (_, bal)| acc + bal);

		let genesis_total_issue = GenesisTotalIssue::<Test>::get();

		assert_eq!(total_issue, genesis_total_issue);
	})
}

#[test]
fn genesis_storage_initialized() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let configured_genesis = genesis_data();

		let mut total_issuence = 0;

		for (who, balance) in configured_genesis.iter() {
			total_issuence += balance;
			assert_eq!(GenesisHolders::<Test>::get(who), Some(*balance));
			assert_eq!(Holdings::<Test>::get(who), Some((*balance, 0)));
		}

		assert_eq!(TotalIssue::<Test>::get(), Some(total_issuence));
	})
}

#[test]
fn mint_hook() {
	let block_number = 11;

	let pool_address = 10;
	let mint_amount = 10;
	new_test_ext().execute_with(|| {
		System::set_block_number(0);

		assert_eq!(
			TotalIssue::<Test>::get(),
			Some(GenesisTotalIssue::<Test>::get())
		);

		assert_eq!(LastMintBlock::<Test>::get(), 0);

		System::set_block_number(block_number);
		AllPalletsWithSystem::on_initialize(block_number);

		assert_eq!(
			TotalIssue::<Test>::get(),
			Some(GenesisTotalIssue::<Test>::get() + mint_amount)
		);

		assert_eq!(Holdings::<Test>::get(pool_address).unwrap().0, mint_amount);

		assert_eq!(
			LastMintBlock::<Test>::get(),
			Pallet::<Test>::block_to_epoch(block_number)
		);
	})
}

#[test]
fn basic_claim() {
	let alice = 1;
	let block_number = 11;
	new_test_ext().execute_with(|| {
		System::set_block_number(block_number);
		AllPalletsWithSystem::on_initialize(block_number);

		assert_ok!(TokenDistribution::claim_pool_reward(RuntimeOrigin::signed(
			alice
		)),);
	})
}

#[test]
fn claim_scenario_example() {
	let alice = 1;
	let alice_first_balance = 500;
	let alice_balance_25th_block_claim = 510;
	let alice_balance_85th_block_claim = 540;

	let bob = 2;
	let bob_first_balance = 500;
	let bob_balance_45th_block_claim = 520;
	let bob_balance_85th_block_claim = 540;

	let pool_address = 10;
	let pool_balance_25th_block = 20;
	let reward_on_pool_at_25_after_claim = 10;
	let reward_on_pool_at_45 = 30;
	let reward_on_pool_at_45_after_claim = 10;
	let reward_on_pool_at_85 = 50;
	let reward_on_pool_at_85_after_claim = 0;
	new_test_ext().execute_with(|| {
		System::set_block_number(0);
		AllPalletsWithSystem::on_initialize(0);

		// Initial Distribution:
		// Wallet 1: 500 tokens
		// Wallet 2: 500 tokens
		assert_eq!(Holdings::<Test>::get(alice).unwrap().0, alice_first_balance);
		assert_eq!(Holdings::<Test>::get(bob).unwrap().0, bob_first_balance);

		for i in 1..=25 {
			System::set_block_number(i);
			AllPalletsWithSystem::on_initialize(i);
		}

		// After 25 blocks:
		//    There are 20 tokens in the reward pool.
		//    Wallet 1 claims their tokens and now has 510 tokens.
		//    10 tokens remain in the reward pool.

		assert_eq!(
			Holdings::<Test>::get(pool_address).unwrap().0,
			pool_balance_25th_block
		);

		assert_eq!(Holdings::<Test>::get(alice).unwrap().0, alice_first_balance);

		assert_ok!(TokenDistribution::claim_pool_reward(RuntimeOrigin::signed(
			alice
		)));

		System::set_block_number(26);

		assert_eq!(
			Holdings::<Test>::get(alice).unwrap().0,
			alice_balance_25th_block_claim
		);

		assert_eq!(
			Holdings::<Test>::get(pool_address).unwrap().0,
			reward_on_pool_at_25_after_claim
		);

		// After a total of 45 blocks:
		//     There are 30 tokens in the reward pool.
		//     Wallet 2 claims their tokens and now has 520 tokens.
		//     10 tokens remain in the reward pool.

		for i in 27..=45 {
			System::set_block_number(i);
			AllPalletsWithSystem::on_initialize(i);
		}

		assert_eq!(
			Holdings::<Test>::get(pool_address).unwrap().0,
			reward_on_pool_at_45
		);

		assert_ok!(TokenDistribution::claim_pool_reward(RuntimeOrigin::signed(
			bob
		)));

		System::set_block_number(46);

		assert_eq!(
			Holdings::<Test>::get(bob).unwrap().0,
			bob_balance_45th_block_claim
		);

		assert_eq!(
			Holdings::<Test>::get(pool_address).unwrap().0,
			reward_on_pool_at_45_after_claim
		);

		// After a total of 85 blocks:
		//     There are 50 tokens in the reward pool.
		//     Both Wallet 1 and Wallet 2 claim their tokens.
		//     Each wallet now has 540 tokens, and the reward pool is empty.

		for i in 47..=85 {
			System::set_block_number(i);
			AllPalletsWithSystem::on_initialize(i);
		}

		assert_eq!(
			Holdings::<Test>::get(pool_address).unwrap().0,
			reward_on_pool_at_85
		);

		assert_ok!(TokenDistribution::claim_pool_reward(RuntimeOrigin::signed(
			alice
		)));

		assert_ok!(TokenDistribution::claim_pool_reward(RuntimeOrigin::signed(
			bob
		)));

		System::set_block_number(86);

		assert_eq!(
			Holdings::<Test>::get(alice).unwrap().0,
			alice_balance_85th_block_claim
		);

		assert_eq!(
			Holdings::<Test>::get(bob).unwrap().0,
			bob_balance_85th_block_claim
		);

		assert_eq!(
			Holdings::<Test>::get(pool_address).unwrap().0,
			reward_on_pool_at_85_after_claim
		);
	})
}

// TODO: testing scenarios
// test mint filed all edge cases
// let current_holding = Holdings::<Test>::get(alice).unwrap().0;

// test to  mint on same epoch
// test mint event is emitted
// last mint on middle of epoch
