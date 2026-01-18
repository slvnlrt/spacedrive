//! Windows-specific volume detection helpers

use crate::volume::{
	classification::{get_classifier, VolumeDetectionInfo},
	error::{VolumeError, VolumeResult},
	types::{DiskType, FileSystem, MountType, Volume, VolumeDetectionConfig, VolumeFingerprint},
	utils,
};
use std::path::PathBuf;
use sysinfo::Disks;
use tokio::task;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Windows volume information from sysinfo
#[derive(Debug, Clone)]
pub struct WindowsVolumeInfo {
	pub drive_letter: Option<String>,
	pub label: Option<String>,
	pub size: u64,
	pub size_remaining: u64,
	pub filesystem: String,
	pub volume_guid: Option<String>,
}

/// Detect Windows volumes using sysinfo crate (cross-platform, native Rust)
pub async fn detect_volumes(
	device_id: Uuid,
	config: &VolumeDetectionConfig,
) -> VolumeResult<Vec<Volume>> {
	let config = config.clone();
	task::spawn_blocking(move || detect_volumes_sysinfo(device_id, &config))
		.await
		.map_err(|e| VolumeError::platform(format!("Task join error: {}", e)))?
}

/// Detect volumes using the sysinfo crate (proper Rust approach)
fn detect_volumes_sysinfo(
	device_id: Uuid,
	_config: &VolumeDetectionConfig,
) -> VolumeResult<Vec<Volume>> {
	let mut volumes = Vec::new();
	let disks = Disks::new_with_refreshed_list();

	info!("sysinfo detected {} disks", disks.list().len());

	for disk in disks.list() {
		let mount_point = disk.mount_point().to_path_buf();
		let name = disk.name().to_string_lossy().to_string();
		let total_bytes = disk.total_space();
		let available_bytes = disk.available_space();
		let filesystem_str = disk.file_system().to_string_lossy().to_string();
		let is_removable = disk.is_removable();
		let disk_kind = disk.kind();

		// Skip if no mount point or zero size
		if mount_point.as_os_str().is_empty() || total_bytes == 0 {
			debug!("Skipping disk with empty mount point or zero size");
			continue;
		}

		info!(
			"Processing disk: {:?} at {:?} ({}), {} bytes, removable={}",
			name, mount_point, filesystem_str, total_bytes, is_removable
		);

		// Parse filesystem type
		let file_system = utils::parse_filesystem_type(&filesystem_str);

		// Determine mount type
		let mount_type = if is_removable {
			MountType::External
		} else {
			// Check if it's the primary drive (usually C:\ on Windows)
			let mount_str = mount_point.to_string_lossy().to_uppercase();
			if mount_str.starts_with("C:") {
				MountType::System
			} else {
				MountType::User
			}
		};

		// Determine disk type from sysinfo
		let disk_type = match disk_kind {
			sysinfo::DiskKind::HDD => DiskType::HDD,
			sysinfo::DiskKind::SSD => DiskType::SSD,
			_ => DiskType::Unknown,
		};

		// Classify volume
		let volume_type = classify_volume(&mount_point, &file_system, &name, total_bytes, is_removable);

		// Generate stable fingerprint based on volume type
		let fingerprint = match volume_type {
			crate::volume::types::VolumeType::External => {
				// Try to read/create dotfile for external volumes
				if let Some(spacedrive_id) =
					utils::read_or_create_dotfile_sync(&mount_point, device_id, None)
				{
					VolumeFingerprint::from_external_volume(spacedrive_id, device_id)
				} else {
					// Fallback to mount_point + device_id for read-only external volumes
					VolumeFingerprint::from_primary_volume(&mount_point, device_id)
				}
			}
			crate::volume::types::VolumeType::Network => {
				// Use mount path as backend identifier for network volumes
				let mount_path_str = mount_point.to_string_lossy();
				// Normalize path for consistent fingerprinting (lowercase on Windows)
				let normalized_path = if cfg!(windows) {
					mount_path_str.to_lowercase()
				} else {
					mount_path_str.to_string()
				};
				VolumeFingerprint::from_network_volume(&normalized_path, &normalized_path)
			}
			_ => {
				// Primary, UserData, Secondary, System, Virtual, Unknown
				// All use stable mount_point + device_id
				VolumeFingerprint::from_primary_volume(&mount_point, device_id)
			}
		};

		info!("Generated fingerprint {} for volume {:?} ({:?})", fingerprint.short_id(), name, mount_point);

		// Use name if available, otherwise generate from mount point
		let display_name = if name.is_empty() {
			let mount_str = mount_point.to_string_lossy();
			if mount_str.len() >= 2 {
				format!("Local Disk ({})", &mount_str[..2])
			} else {
				"Local Disk".to_string()
			}
		} else {
			name.clone()
		};

		let mut volume = Volume::new(device_id, fingerprint, display_name, mount_point);

		volume.mount_type = mount_type;
		volume.volume_type = volume_type;
		volume.disk_type = disk_type;
		volume.file_system = file_system;
		volume.total_capacity = total_bytes;
		volume.available_space = available_bytes;
		volume.is_read_only = false; // sysinfo doesn't provide this directly

		volumes.push(volume);
	}

	info!("Detected {} volumes via sysinfo", volumes.len());
	Ok(volumes)
}

/// Classify a volume using the platform-specific classifier
fn classify_volume(
	mount_point: &PathBuf,
	file_system: &FileSystem,
	name: &str,
	total_bytes: u64,
	is_removable: bool,
) -> crate::volume::types::VolumeType {
	let classifier = get_classifier();
	let detection_info = VolumeDetectionInfo {
		mount_point: mount_point.clone(),
		file_system: file_system.clone(),
		total_bytes_capacity: total_bytes,
		is_removable: Some(is_removable),
		is_network_drive: None,  // Would need additional detection
		device_model: None,      // Would need additional detection
	};

	classifier.classify(&detection_info)
}

/// Determine mount type for Windows drives
fn determine_mount_type_windows(drive_letter: &str) -> MountType {
	match drive_letter.to_uppercase().as_str() {
		"C:\\" | "D:\\" => MountType::System, // Common system drives
		_ => MountType::External,             // Assume external for others
	}
}

/// Get Windows volume info using sysinfo (stub kept for API compatibility)
pub async fn get_windows_volume_info() -> VolumeResult<Vec<WindowsVolumeInfo>> {
	// This would be implemented with proper sysinfo parsing
	Ok(Vec::new())
}

/// Create volume from Windows info (stub kept for API compatibility)
pub fn create_volume_from_windows_info(
	info: WindowsVolumeInfo,
	device_id: Uuid,
) -> VolumeResult<Volume> {
	let mount_path = if let Some(drive_letter) = &info.drive_letter {
		PathBuf::from(format!("{}:\\", drive_letter))
	} else {
		PathBuf::from("C:\\") // Default fallback
	};

	let name = info.label.unwrap_or_else(|| {
		if let Some(drive) = &info.drive_letter {
			format!("Local Disk ({}:)", drive)
		} else {
			"Unknown Drive".to_string()
		}
	});

	let file_system = utils::parse_filesystem_type(&info.filesystem);
	let mount_type = if let Some(drive) = &info.drive_letter {
		determine_mount_type_windows(&format!("{}:\\", drive))
	} else {
		MountType::System
	};
	let volume_type = classify_volume(&mount_path, &file_system, &name, info.size, false);

	// Generate stable fingerprint based on volume type
	let fingerprint = match volume_type {
		crate::volume::types::VolumeType::External => {
			// Try to read/create dotfile for external volumes
			if let Some(spacedrive_id) =
				utils::read_or_create_dotfile_sync(&mount_path, device_id, None)
			{
				VolumeFingerprint::from_external_volume(spacedrive_id, device_id)
			} else {
				// Fallback to mount_point + device_id for read-only external volumes
				VolumeFingerprint::from_primary_volume(&mount_path, device_id)
			}
		}
		crate::volume::types::VolumeType::Network => {
			// Use mount path as backend identifier for network volumes
			let mount_path_str = mount_path.to_string_lossy();
			let backend_id = info.volume_guid.as_deref().unwrap_or(&mount_path_str);
			VolumeFingerprint::from_network_volume(backend_id, &mount_path_str)
		}
		_ => {
			// Primary, UserData, Secondary, System, Virtual, Unknown
			// All use stable mount_point + device_id
			VolumeFingerprint::from_primary_volume(&mount_path, device_id)
		}
	};

	let mut volume = Volume::new(device_id, fingerprint, name.clone(), mount_path);

	volume.mount_type = mount_type;
	volume.volume_type = volume_type;
	volume.disk_type = DiskType::Unknown;
	volume.file_system = file_system;
	volume.total_capacity = info.size;
	volume.available_space = info.size_remaining;
	volume.is_read_only = false;
	volume.hardware_id = info.volume_guid;

	Ok(volume)
}

/// Check if volume should be included based on config
pub fn should_include_volume(volume: &Volume, config: &VolumeDetectionConfig) -> bool {
	// Apply filtering based on config
	if !config.include_system && matches!(volume.mount_type, MountType::System) {
		return false;
	}

	// FIX: Use parentheses to call the method
	if !config.include_virtual && volume.total_bytes_capacity() == 0 {
		return false;
	}

	true
}
