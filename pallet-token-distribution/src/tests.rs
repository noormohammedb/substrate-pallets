use crate::{mock::*, *};

#[test]
fn test_genesis_total_issue() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let configured_genesis = genesis_data();

		let total_issue = configured_genesis.iter().fold(0, |acc, (_, bal)| acc + bal);

		let genesis_total_issue = GenesisTotalIssue::<Test>::get();

		assert_eq!(total_issue, genesis_total_issue);
	})
}
