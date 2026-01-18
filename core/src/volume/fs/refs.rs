//! ReFS filesystem-specific detection and optimization
//!
//! This module handles ReFS volume detection and provides ReFS-specific
//! optimizations like block cloning operations.

use crate::volume::{error::VolumeResult, types::Volume};
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use tokio::task;
use tracing::{debug, warn};
use std::sync::Mutex;
use std::collections::HashMap;

#[cfg(windows)]
static REFS_CACHE: Mutex<Option<HashMap<PathBuf, bool>>> = Mutex::new(None);

/// ReFS filesystem handler
pub struct RefsHandler;

impl RefsHandler {
	pub fn new() -> Self {
		Self
	}

	/// Check if two paths are on the same ReFS volume and support block cloning
	pub async fn same_physical_storage(&self, path1: &Path, path2: &Path) -> bool {
		// Check if both paths are on the same ReFS volume
		if let (Ok(vol1), Ok(vol2)) = (
			self.get_volume_info(path1).await,
			self.get_volume_info(path2).await,
		) {
			// Same volume GUID = same physical storage
			return vol1.volume_guid == vol2.volume_guid
				&& vol1.supports_block_cloning
				&& vol2.supports_block_cloning;
		}

		false
	}

	/// Get ReFS volume information for a path
	async fn get_volume_info(&self, path: &Path) -> VolumeResult<RefsVolumeInfo> {
		let path = path.to_path_buf();

		task::spawn_blocking(move || {
			let disks = sysinfo::Disks::new_with_refreshed_list();
			let mut best_match: Option<(&sysinfo::Disk, usize)> = None;

			// Find the disk with the longest mount point that matches the path prefix
			for disk in &disks {
				let mount_point = disk.mount_point();
				if path.starts_with(mount_point) {
					let len = mount_point.as_os_str().len();
					if best_match.map_or(true, |(_, l)| len > l) {
						best_match = Some((disk, len));
					}
				}
			}

			if let Some((disk, _)) = best_match {
				let mount_str = disk.mount_point().to_string_lossy();
				Ok(RefsVolumeInfo {
					volume_guid: mount_str.to_string(), // Fallback: use mount point as ID
					file_system: disk.file_system().to_string_lossy().to_string(),
					drive_letter: mount_str.chars().next(),
					label: Some(disk.name().to_string_lossy().to_string()),
					size_bytes: disk.total_space(),
					available_bytes: disk.available_space(),
					disk_number: None,
					partition_number: None,
					media_type: Some(if disk.is_removable() {
						"Removable".to_string()
					} else {
						"Fixed".to_string()
					}),
					supports_block_cloning: disk.file_system().to_string_lossy() == "ReFS",
				})
			} else {
				Err(crate::volume::error::VolumeError::NotFound(format!(
					"No volume found for path {}",
					path.display()
				)))
			}
		})
		.await
		.map_err(|e| crate::volume::error::VolumeError::platform(format!("Task join error: {}", e)))?
	}

	/// Check if ReFS block cloning is supported
	#[cfg(windows)]
	async fn supports_block_cloning(&self, path: &Path) -> bool {
		use std::mem::size_of;
		use std::os::windows::ffi::OsStrExt;
		use std::ptr::{null_mut, null};
		use windows_sys::Win32::Foundation::{CloseHandle, GENERIC_READ, INVALID_HANDLE_VALUE};
		use windows_sys::Win32::Storage::FileSystem::{
			CreateFileW, FILE_FLAG_BACKUP_SEMANTICS, FILE_SHARE_READ, FILE_SHARE_WRITE,
			OPEN_EXISTING,
		};
		use windows_sys::Win32::System::Ioctl::{FSCTL_GET_REFS_VOLUME_DATA, REFS_VOLUME_DATA_BUFFER};
		use windows_sys::Win32::System::IO::DeviceIoControl;

		let path_buf = path.to_path_buf();

		// Check cache first
		if let Ok(guard) = REFS_CACHE.lock() {
			if let Some(cache) = guard.as_ref() {
				if let Some(&supported) = cache.get(&path_buf) {
					return supported;
				}
			}
		}

		// Run blocking IO operation in a dedicated thread
		task::spawn_blocking(move || {
			let wide_path: Vec<u16> = path_buf.as_os_str().encode_wide().chain(Some(0)).collect();

			// Open volume handle
			// FILE_FLAG_BACKUP_SEMANTICS is required to open a directory handle
			let handle = unsafe {
				CreateFileW(
					wide_path.as_ptr(),
					GENERIC_READ,
					FILE_SHARE_READ | FILE_SHARE_WRITE,
					null(), // lpSecurityAttributes
					OPEN_EXISTING,
					FILE_FLAG_BACKUP_SEMANTICS,
					0, // hTemplateFile (HANDLE is isize, use 0)
				)
			};

			if handle == INVALID_HANDLE_VALUE {
				warn!("Failed to open volume handle for ReFS check: {}", path_buf.display());
				return false;
			}

			let mut buffer: REFS_VOLUME_DATA_BUFFER = unsafe { std::mem::zeroed() };
			let mut bytes_returned = 0u32;

			let result = unsafe {
				DeviceIoControl(
					handle,
					FSCTL_GET_REFS_VOLUME_DATA,
					null(),
					0,
					&mut buffer as *mut _ as *mut _,
					size_of::<REFS_VOLUME_DATA_BUFFER>() as u32,
					&mut bytes_returned,
					null_mut(),
				)
			};

			unsafe {
				CloseHandle(handle);
			}

			if result == 0 {
				warn!("FSCTL_GET_REFS_VOLUME_DATA failed for: {}", path_buf.display());
				return false;
			}

			// ReFS v2.0+ (Windows Server 2016 / Windows 10 1703) supports block cloning
			// MajorVersion 2+ guarantees support.
			let supported = buffer.MajorVersion >= 2;
			debug!("ReFS version {}.{} on {}: Block cloning supported = {}",
				buffer.MajorVersion, buffer.MinorVersion, path_buf.display(), supported);

			// Update cache
			if let Ok(mut guard) = REFS_CACHE.lock() {
				if guard.is_none() {
					*guard = Some(HashMap::new());
				}
				if let Some(cache) = guard.as_mut() {
					cache.insert(path_buf.clone(), supported);
				}
			}

			supported
		})
		.await
		.unwrap_or(false)
	}

	#[cfg(not(windows))]
	async fn supports_block_cloning(&self, _path: &Path) -> bool {
		false
	}

	/// Get all ReFS volumes on the system
	pub async fn get_all_refs_volumes(&self) -> VolumeResult<Vec<RefsVolumeInfo>> {
		task::spawn_blocking(|| {
			let disks = sysinfo::Disks::new_with_refreshed_list();
			let mut refs_volumes = Vec::new();

			for disk in &disks {
				let fs_name = disk.file_system().to_string_lossy();
				if fs_name == "ReFS" {
					let mount_point = disk.mount_point().to_string_lossy();
					refs_volumes.push(RefsVolumeInfo {
						volume_guid: mount_point.to_string(), // Fallback
						file_system: fs_name.to_string(),
						drive_letter: mount_point.chars().next(),
						label: Some(disk.name().to_string_lossy().to_string()),
						size_bytes: disk.total_space(),
						available_bytes: disk.available_space(),
						disk_number: None,
						partition_number: None,
						media_type: Some(if disk.is_removable() {
							"Removable".to_string()
						} else {
							"Fixed".to_string()
						}),
						supports_block_cloning: true, // ReFS implies potential support, detail check later
					});
				}
			}

			Ok(refs_volumes)
		})
		.await
		.map_err(|e| crate::volume::error::VolumeError::platform(format!("Task join error: {}", e)))?
	}
}

#[async_trait]
impl super::FilesystemHandler for RefsHandler {
	async fn enhance_volume(&self, volume: &mut Volume) -> VolumeResult<()> {
		// Add ReFS-specific information like block cloning support
		if let Some(mount_point) = volume.mount_point.to_str() {
			if self.supports_block_cloning(Path::new(mount_point)).await {
				debug!("ReFS volume supports block cloning: {}", mount_point);
				// Could store this capability in volume metadata
			}
		}
		Ok(())
	}

	async fn same_physical_storage(&self, path1: &Path, path2: &Path) -> bool {
		self.same_physical_storage(path1, path2).await
	}

	fn get_copy_strategy(&self) -> Box<dyn crate::ops::files::copy::strategy::CopyStrategy> {
		// Use fast copy strategy for ReFS (leverages block cloning)
		Box::new(crate::ops::files::copy::strategy::FastCopyStrategy)
	}

	fn contains_path(&self, volume: &Volume, path: &std::path::Path) -> bool {
		// Strip Windows extended path prefix if present
		// This handles canonicalized paths that start with \\?\
		let normalized_path = if let Some(path_str) = path.to_str() {
			if path_str.starts_with("\\\\?\\UNC\\") {
				PathBuf::from(format!("\\\\{}", &path_str[8..]))
			} else if let Some(stripped) = path_str.strip_prefix("\\\\?\\") {
				PathBuf::from(stripped)
			} else {
				path.to_path_buf()
			}
		} else {
			path.to_path_buf()
		};

		// Check primary mount point
		if normalized_path.starts_with(&volume.mount_point) {
			return true;
		}

		// Check additional mount points
		if volume.mount_points.iter().any(|mp| normalized_path.starts_with(mp)) {
			return true;
		}

		// TODO: ReFS-specific logic for junction points and mount points
		// Windows can have volumes mounted as folders (mount points) within other volumes
		// This would require checking Windows-specific mount point resolution

		false
	}
}

/// ReFS volume information
#[derive(Debug, Clone)]
pub struct RefsVolumeInfo {
	pub volume_guid: String,
	pub file_system: String,
	pub drive_letter: Option<char>,
	pub label: Option<String>,
	pub size_bytes: u64,
	pub available_bytes: u64,
	pub disk_number: Option<u32>,
	pub partition_number: Option<u32>,
	pub media_type: Option<String>,
	pub supports_block_cloning: bool,
}

/// Enhance volume with ReFS-specific information from Windows
pub async fn enhance_volume_from_windows(volume: &mut Volume) -> VolumeResult<()> {
	// Import the trait from the parent module so the enhance_volume method is available
	use super::FilesystemHandler;

	let handler = RefsHandler::new();
	handler.enhance_volume(volume).await
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_refs_volume_info_creation() {
		let info = RefsVolumeInfo {
			volume_guid: "E:\\".to_string(),
			file_system: "ReFS".to_string(),
			drive_letter: Some('E'),
			label: Some("DevDrive".to_string()),
			size_bytes: 100_000_000_000,
			available_bytes: 50_000_000_000,
			disk_number: None,
			partition_number: None,
			media_type: Some("Fixed".to_string()),
			supports_block_cloning: true,
		};

		assert_eq!(info.file_system, "ReFS");
		assert!(info.supports_block_cloning);
		assert_eq!(info.drive_letter, Some('E'));
	}

	#[test]
	fn test_refs_handler_creation() {
		let handler = RefsHandler::new();
		// Handler should be created without panicking
		let _ = handler;
	}
}
