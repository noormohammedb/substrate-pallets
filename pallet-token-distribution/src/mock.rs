use crate as pallet_token_distribution;
use polkadot_sdk_frame::deps::frame_support::{derive_impl, runtime};
use polkadot_sdk_frame::deps::sp_io;

use polkadot_sdk_frame::prelude::*;
use polkadot_sdk_frame::testing_prelude::BuildStorage;
use polkadot_sdk_frame::traits::{ConstU128, ConstU64};

type Block = frame_system::mocking::MockBlock<Test>;
type Balance = u64;

#[runtime]
mod Test {

	#[runtime::runtime]
	#[runtime::derive(
		RuntimeCall,
		RuntimeEvent,
		RuntimeError,
		RuntimeOrigin,
		RuntimeFreezeReason,
		RuntimeHoldReason,
		RuntimeTask
	)]
	pub struct Test;

	#[runtime::pallet_index(0)]
	pub type System = frame_system;

	#[runtime::pallet_index(1)]
	pub type Balances = pallet_balances;

	#[runtime::pallet_index(2)]
	pub type TokenDistribution = pallet_token_distribution;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
	type Block = Block;
	type AccountData = pallet_balances::AccountData<Balance>;
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for Test {
	type AccountStore = System;
}

impl pallet_token_distribution::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type EpochPeriod = ConstU64<10>;
	type MintAmountPerEpoch = ConstU128<10>;
	type PoolAddress = ConstU64<10>;
	//
}

pub fn genesis_data() -> Vec<(u64, u128)> {
	vec![(1, 150), (2, 200), (3, 250)]
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	RuntimeGenesisConfig {
		token_distribution: crate::pallet::GenesisConfig::<Test> {
			initial_token_distribution: genesis_data(),
		},
		..Default::default()
	}
	.build_storage()
	.unwrap()
	.into()
}
