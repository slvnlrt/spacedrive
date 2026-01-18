//! Generic filesystem handler for unknown/unsupported filesystems

use super::FilesystemHandler;
use crate::volume::{error::VolumeResult, types::Volume};
use async_trait::async_trait;

/// Generic handler for filesystems without specific optimizations
pub struct GenericFilesystemHandler;

#[async_trait]
impl FilesystemHandler for GenericFilesystemHandler {
	async fn enhance_volume(&self, _volume: &mut Volume) -> VolumeResult<()> {
		// No special enhancements for generic filesystems
		Ok(())
	}

	async fn same_physical_storage(
		&self,
		path1: &std::path::Path,
		path2: &std::path::Path,
	) -> bool {
		// For generic filesystems, we can only check if they resolve to the same device
		// This is a conservative approach that may miss some optimizations
		if let (Ok(meta1), Ok(meta2)) = (path1.metadata(), path2.metadata()) {
			// On Unix systems, compare device IDs
			#[cfg(unix)]
			{
				use std::os::unix::fs::MetadataExt;
				return meta1.dev() == meta2.dev();
			}

			// On Windows, this is more complex and would need additional logic
			#[cfg(windows)]
			{
				// For now, be conservative and assume different storage
				return false;
			}
		}

		false
	}

	fn get_copy_strategy(&self) -> Box<dyn crate::ops::files::copy::strategy::CopyStrategy> {
		// Use streaming copy as the safe default
		Box::new(crate::ops::files::copy::strategy::LocalStreamCopyStrategy)
	}

	fn contains_path(&self, volume: &Volume, path: &std::path::Path) -> bool {
		// Strip Windows extended path prefix if present
		// This handles canonicalized paths that start with \\?\ (harmless on other platforms)
		let normalized_path = if let Some(path_str) = path.to_str() {
			if path_str.starts_with("\\\\?\\UNC\\") {
				std::path::PathBuf::from(format!("\\\\{}", &path_str[8..]))
			} else if let Some(stripped) = path_str.strip_prefix("\\\\?\\") {
				std::path::PathBuf::from(stripped)
			} else {
				path.to_path_buf()
			}
		} else {
			path.to_path_buf()
		};

		// Generic implementation: only check mount points
		// Check primary mount point
		if normalized_path.starts_with(&volume.mount_point) {
			return true;
		}

		// Check additional mount points
		volume.mount_points.iter().any(|mp| normalized_path.starts_with(mp))
	}
}
