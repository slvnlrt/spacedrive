//! Volume untrack action
//!
//! This action untracks a volume from the library. It delegates to
//! `VolumeManager::untrack_volume_by_id()` to ensure consistent behavior:
//! - Database record is deleted
//! - In-memory cache is updated (is_tracked = false)
//! - ResourceChanged event is emitted (NOT ResourceDeleted, since the volume still exists)
//!
//! The distinction matters: untracking means Spacedrive will stop indexing/managing
//! the volume, but the physical volume still exists and should remain visible in the UI
//! with an "untracked" indicator.

use super::{VolumeUntrackInput, VolumeUntrackOutput};
use crate::{context::CoreContext, infra::action::error::ActionError};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeUntrackAction {
	input: VolumeUntrackInput,
}

impl VolumeUntrackAction {
	pub fn new(input: VolumeUntrackInput) -> Self {
		Self { input }
	}
}

crate::register_library_action!(VolumeUntrackAction, "volumes.untrack");

impl crate::infra::action::LibraryAction for VolumeUntrackAction {
	type Input = VolumeUntrackInput;
	type Output = VolumeUntrackOutput;

	fn from_input(input: Self::Input) -> Result<Self, String> {
		Ok(VolumeUntrackAction::new(input))
	}

	async fn execute(
		self,
		library: Arc<crate::library::Library>,
		context: Arc<CoreContext>,
	) -> Result<Self::Output, ActionError> {
		// Delegate to VolumeManager for consistent behavior
		// This ensures:
		// 1. Database record is properly deleted
		// 2. In-memory cache is updated (is_tracked = false, library_id = None)
		// 3. VolumeRemoved event is emitted for internal listeners
		// 4. ResourceChanged event is emitted for UI reactivity (NOT ResourceDeleted!)
		context
			.volume_manager
			.untrack_volume_by_id(&library, self.input.volume_id)
			.await
			.map_err(|e| ActionError::Internal(e.to_string()))?;

		Ok(VolumeUntrackOutput {
			volume_id: self.input.volume_id,
			success: true,
		})
	}

	fn action_kind(&self) -> &'static str {
		"volumes.untrack"
	}
}
