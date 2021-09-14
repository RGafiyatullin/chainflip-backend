use crate::{RotationError, VaultRotation};
use std::cell::RefCell;

thread_local! {
	pub static TO_CONFIRM: RefCell<Result<(), RotationError<u64>>> = RefCell::new(Err(RotationError::NotConfirmed));
}

pub struct Mock {}

// Helper function to clear the confirmation result
pub fn clear_confirmation() {
	TO_CONFIRM.with(|l| *l.borrow_mut() = Ok(()));
}

impl VaultRotation for Mock {
	type AccountId = u64;

	fn start_vault_rotation(
		_candidates: Vec<Self::AccountId>,
	) -> Result<(), RotationError<Self::AccountId>> {
		TO_CONFIRM.with(|l| *l.borrow_mut() = Err(RotationError::NotConfirmed));
		Ok(())
	}

	fn finalize_rotation() -> Result<(), RotationError<Self::AccountId>> {
		TO_CONFIRM.with(|l| (*l.borrow()).clone())
	}
}
