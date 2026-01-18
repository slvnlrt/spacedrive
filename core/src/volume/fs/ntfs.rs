//! NTFS filesystem-specific detection and optimization
//!
//! This module handles NTFS volume detection and provides NTFS-specific
//! optimizations like hardlink and junction point handling.

use crate::volume::{error::VolumeResult, types::Volume};
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use tokio::task;
use tracing::{debug, warn};
#[cfg(windows)]
use windows_sys::Win32::Foundation::MAX_PATH;
#[cfg(windows)]
use windows_sys::Win32::Storage::FileSystem::{GetVolumeInformationW, GetVolumePathNameW};
#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;

/// NTFS filesystem handler
pub struct NtfsHandler;

impl NtfsHandler {
	pub fn new() -> Self {
		Self
	}

	/// Check if two paths are on the same NTFS volume
	pub async fn same_physical_storage(&self, path1: &Path, path2: &Path) -> bool {
		// Check if both paths are on the same NTFS volume
		if let (Ok(vol1), Ok(vol2)) = (
			self.get_volume_info(path1).await,
			self.get_volume_info(path2).await,
		) {
			// Same volume GUID = same physical storage
			return vol1.volume_guid == vol2.volume_guid;
		}

		false
	}

	/// Get NTFS volume information for a path
	async fn get_volume_info(&self, path: &Path) -> VolumeResult<NtfsVolumeInfo> {
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
				Ok(NtfsVolumeInfo {
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

	/// Check if NTFS hardlinks are supported (they always are on NTFS)
	pub async fn supports_hardlinks(&self, path: &Path) -> bool {
		// NTFS always supports hardlinks
		if let Ok(vol_info) = self.get_volume_info(path).await {
			return vol_info.file_system.to_uppercase() == "NTFS";
		}
		false
	}

	/// Check if NTFS junction points are supported
	pub async fn supports_junctions(&self, path: &Path) -> bool {
		// NTFS supports junction points (directory symbolic links)
		if let Ok(vol_info) = self.get_volume_info(path).await {
			return vol_info.file_system.to_uppercase() == "NTFS";
		}
		false
	}

	/// Resolve junction points and symbolic links
	pub async fn resolve_ntfs_path(&self, path: &Path) -> PathBuf {
		let path = path.to_path_buf();
		let path_for_closure = path.clone();
		task::spawn_blocking(move || {
			// Use standard library canonicalize
			match std::fs::canonicalize(&path_for_closure) {
				Ok(p) => {
					// Strip \\?\ prefix on Windows
					let s = p.to_string_lossy();
					if s.starts_with(r"\\?\") {
						PathBuf::from(&s[4..])
					} else {
						p
					}
				}
				Err(_) => path_for_closure, // Fallback to original path if resolution fails
			}
		})
		.await
		.unwrap_or(path) // If task join fails, return original path
	}

	/// Get NTFS file system features
	pub async fn get_ntfs_features(&self, path: &Path) -> VolumeResult<NtfsFeatures> {
		let path = path.to_path_buf();
		task::spawn_blocking(move || {
			#[cfg(windows)]
			{
				let path_str = path.as_os_str();
				let wide_path: Vec<u16> = path_str.encode_wide().chain(std::iter::once(0)).collect();
				let mut root_path = [0u16; MAX_PATH as usize];

				// 1. Get volume root path (resolves mount points etc.)
				let success = unsafe {
					GetVolumePathNameW(
						wide_path.as_ptr(),
						root_path.as_mut_ptr(),
						root_path.len() as u32,
					)
				};

				if success == 0 {
					return Err(crate::volume::error::VolumeError::platform(
						"Failed to get volume path name".to_string(),
					));
				}

				// 2. Get volume information
				let mut flags: u32 = 0;
				let success = unsafe {
					GetVolumeInformationW(
						root_path.as_ptr(),
						std::ptr::null_mut(),
						0,
						std::ptr::null_mut(),
						std::ptr::null_mut(),
						&mut flags,
						std::ptr::null_mut(),
						0,
					)
				};

				if success == 0 {
					return Err(crate::volume::error::VolumeError::platform(
						"Failed to get volume information".to_string(),
					));
				}

				// Constants for GetVolumeInformationW bits
				// Constants for GetVolumeInformationW bits
				const FILE_FILE_COMPRESSION: u32 = 0x00000010;
				const FILE_SUPPORTS_REPARSE_POINTS: u32 = 0x00000080;
				const FILE_SUPPORTS_ENCRYPTION: u32 = 0x00020000;
				const FILE_NAMED_STREAMS: u32 = 0x00040000;
				const FILE_SUPPORTS_HARD_LINKS: u32 = 0x00400000;

				Ok(NtfsFeatures {
					supports_hardlinks: (flags & FILE_SUPPORTS_HARD_LINKS) != 0,
					supports_junctions: (flags & FILE_SUPPORTS_REPARSE_POINTS) != 0,
					supports_symlinks: (flags & FILE_SUPPORTS_REPARSE_POINTS) != 0,
					supports_streams: (flags & FILE_NAMED_STREAMS) != 0,
					supports_compression: (flags & FILE_FILE_COMPRESSION) != 0,
					supports_encryption: (flags & FILE_SUPPORTS_ENCRYPTION) != 0,
				})
			}
			#[cfg(not(windows))]
			{
				Ok(NtfsFeatures {
					supports_hardlinks: true,
					supports_junctions: true,
					supports_symlinks: true,
					supports_streams: true,
					supports_compression: true,
					supports_encryption: true,
				})
			}
		})
		.await
		.map_err(|e| crate::volume::error::VolumeError::platform(format!("Task join error: {}", e)))?
	}
}

#[async_trait]
impl super::FilesystemHandler for NtfsHandler {
	async fn enhance_volume(&self, volume: &mut Volume) -> VolumeResult<()> {
		// Add NTFS-specific information like feature support
		if let Some(mount_point) = volume.mount_point.to_str() {
			if let Ok(features) = self.get_ntfs_features(Path::new(mount_point)).await {
				debug!("Enhanced NTFS volume with features: {:?}", features);
				// Could store NTFS features in volume metadata
			}
		}
		Ok(())
	}

	async fn same_physical_storage(&self, path1: &Path, path2: &Path) -> bool {
		self.same_physical_storage(path1, path2).await
	}

	fn get_copy_strategy(&self) -> Box<dyn crate::ops::files::copy::strategy::CopyStrategy> {
		// Use streaming copy for NTFS (no built-in CoW like APFS/ReFS)
		// Could potentially use hardlinks for same-volume copies
		Box::new(crate::ops::files::copy::strategy::LocalStreamCopyStrategy)
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

		// TODO: NTFS-specific logic for junction points and mount points
		// Windows can have volumes mounted as folders (mount points) within other volumes
		// NTFS also supports junction points and symbolic links that may need resolution

		false
	}
}

/// NTFS volume information
#[derive(Debug, Clone)]
pub struct NtfsVolumeInfo {
	pub volume_guid: String,
	pub file_system: String,
	pub drive_letter: Option<char>,
	pub label: Option<String>,
	pub size_bytes: u64,
	pub available_bytes: u64,
	pub disk_number: Option<u32>,
	pub partition_number: Option<u32>,
	pub media_type: Option<String>,
}

/// NTFS filesystem features
#[derive(Debug, Clone)]
pub struct NtfsFeatures {
	pub supports_hardlinks: bool,
	pub supports_junctions: bool,
	pub supports_symlinks: bool,
	pub supports_streams: bool,
	pub supports_compression: bool,
	pub supports_encryption: bool,
}




/// Enhance volume with NTFS-specific information from Windows
pub async fn enhance_volume_from_windows(volume: &mut Volume) -> VolumeResult<()> {
	// FIX: Import the trait from the correct module
	use crate::volume::fs::FilesystemHandler;

	let handler = NtfsHandler::new();
	handler.enhance_volume(volume).await
}

#[cfg(test)]
mod tests {
	use super::*;
	// Tests for extract_json_string removed as the function was replaced by sysinfo
}
