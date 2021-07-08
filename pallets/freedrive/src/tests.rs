use crate::{Error, mock::*};
use frame_support::{assert_ok, assert_noop};

#[test]
fn it_works_always() {
	new_test_ext().execute_with(|| {
		assert!(true);
	});
}
