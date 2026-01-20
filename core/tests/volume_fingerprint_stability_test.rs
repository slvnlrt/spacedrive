use sd_core::domain::volume::VolumeFingerprint;
use std::path::Path;
use uuid::Uuid;

/// Test that identical volume properties produce identical fingerprints
#[test]
fn test_fingerprint_deterministic() {
	let mount_point = Path::new(if cfg!(windows) { "C:\\" } else { "/" });
	let device_id = Uuid::new_v4();

	// Generate fingerprint multiple times with same inputs
	let fp1 = VolumeFingerprint::from_primary_volume(mount_point, device_id);
	let fp2 = VolumeFingerprint::from_primary_volume(mount_point, device_id);
	let fp3 = VolumeFingerprint::from_primary_volume(mount_point, device_id);

	assert_eq!(fp1, fp2, "Fingerprints should be identical for same inputs");
	assert_eq!(fp2, fp3, "Fingerprints should be deterministic");

	println!("Fingerprint is deterministic: {}", fp1.short_id());
}

/// Test that different stable identifiers produce different fingerprints
#[test]
fn test_fingerprint_changes_with_identifiers() {
	let mount_c = Path::new("C:\\");
	let mount_d = Path::new("D:\\");
	let device_1 = Uuid::new_v4();
	let device_2 = Uuid::new_v4();

	let fp_c = VolumeFingerprint::from_primary_volume(mount_c, device_1);
	let fp_d = VolumeFingerprint::from_primary_volume(mount_d, device_1);
	let fp_dev2 = VolumeFingerprint::from_primary_volume(mount_c, device_2);

	assert_ne!(
		fp_c, fp_d,
		"Different mount points should produce different fingerprints"
	);
	assert_ne!(
		fp_c, fp_dev2,
		"Different device IDs should produce different fingerprints"
	);

	println!("C: fingerprint: {}", fp_c.short_id());
	println!("D: fingerprint: {}", fp_d.short_id());
}

/// Test external volume stability
#[test]
fn test_external_volume_fingerprint_stability() {
	let spacedrive_id = Uuid::new_v4();
	let device_id = Uuid::new_v4();

	let fp1 = VolumeFingerprint::from_external_volume(spacedrive_id, device_id);
	let fp2 = VolumeFingerprint::from_external_volume(spacedrive_id, device_id);

	assert_eq!(
		fp1, fp2,
		"External fingerprints should be stable for same IDs"
	);

	let different_device = Uuid::new_v4();
	let fp3 = VolumeFingerprint::from_external_volume(spacedrive_id, different_device);

	assert_ne!(fp1, fp3, "External fingerprints should bind to device ID");
}

/// Test normalization of paths for primary volumes
#[test]
fn test_fingerprint_path_normalization() {
	let device_id = Uuid::new_v4();

	let fp1 = VolumeFingerprint::from_primary_volume(Path::new("/Volumes/Data/"), device_id);
	let fp2 = VolumeFingerprint::from_primary_volume(Path::new("/Volumes/Data"), device_id);

	assert_eq!(fp1, fp2, "Trailing slashes should be normalized");

	if cfg!(windows) || cfg!(target_os = "macos") {
		let fp_lower = VolumeFingerprint::from_primary_volume(Path::new("C:\\Data"), device_id);
		let fp_upper = VolumeFingerprint::from_primary_volume(Path::new("c:\\data"), device_id);
		assert_eq!(
			fp_lower, fp_upper,
			"Paths should be case-insensitive on this OS"
		);
	}
}

/// Test actual volume detection and fingerprint consistency
#[cfg(target_os = "macos")]
#[tokio::test]
async fn test_real_volume_fingerprints_remain_stable() {
	println!("\nTesting real volume fingerprint stability...\n");

	// Initialize volume manager
	let device_id = Uuid::new_v4();
	let config = VolumeDetectionConfig::default();
	let events = Arc::new(EventBus::default());
	let volume_manager = Arc::new(VolumeManager::new(device_id, config, events));

	// First detection
	volume_manager
		.initialize()
		.await
		.expect("Failed to initialize volume manager");

	let volumes_first = volume_manager.get_all_volumes().await;
	let mut first_fingerprints: HashMap<String, String> = HashMap::new();

	println!("First detection - {} volumes:", volumes_first.len());
	for volume in &volumes_first {
		let fp_string = volume.fingerprint.to_string();
		first_fingerprints.insert(
			volume.mount_point.to_string_lossy().to_string(),
			fp_string.clone(),
		);

		println!(
			"  {} → fingerprint: {}",
			volume.mount_point.display(),
			volume.fingerprint.short_id()
		);

		// Print what went into the fingerprint
		if let Some(container) = &volume.apfs_container {
			println!(
				"    (container: {}, capacity: {} bytes)",
				container.uuid, container.total_capacity
			);
		}
	}

	// Small delay to simulate time passing
	thread::sleep(Duration::from_millis(100));

	// Re-detect volumes (simulating daemon restart)
	let volume_manager2 = Arc::new(VolumeManager::new(
		device_id,
		VolumeDetectionConfig::default(),
		Arc::new(EventBus::default()),
	));

	volume_manager2
		.initialize()
		.await
		.expect("Failed to initialize volume manager (second time)");

	let volumes_second = volume_manager2.get_all_volumes().await;

	println!("\nSecond detection - {} volumes:", volumes_second.len());

	let mut stable_count = 0;
	let mut changed_count = 0;

	for volume in &volumes_second {
		let mount_point = volume.mount_point.to_string_lossy().to_string();
		let fp_string = volume.fingerprint.to_string();

		println!(
			"  {} → fingerprint: {}",
			volume.mount_point.display(),
			volume.fingerprint.short_id()
		);

		if let Some(first_fp) = first_fingerprints.get(&mount_point) {
			if first_fp == &fp_string {
				println!("    STABLE - fingerprint unchanged");
				stable_count += 1;
			} else {
				println!("    CHANGED - fingerprint different!");
				println!("       Was: {}", first_fp);
				println!("       Now: {}", fp_string);
				changed_count += 1;
			}
		} else {
			println!("    ️  New volume (not in first detection)");
		}
	}

	println!("\nResults:");
	println!("  Stable: {}", stable_count);
	println!("  Changed: {}", changed_count);

	assert_eq!(
		changed_count, 0,
		"All volume fingerprints should remain stable across detections"
	);
}

/// Test what properties actually change vs stay stable
#[cfg(target_os = "macos")]
#[tokio::test]
async fn test_what_properties_change_on_real_volumes() {
	println!("\nAnalyzing which volume properties change over time...\n");

	let device_id = Uuid::new_v4();
	let config = VolumeDetectionConfig::default();
	let events = Arc::new(EventBus::default());
	let volume_manager = Arc::new(VolumeManager::new(device_id, config, events));

	volume_manager
		.initialize()
		.await
		.expect("Failed to initialize volume manager");

	let volumes = volume_manager.get_all_volumes().await;

	println!("Analyzing {} volumes:\n", volumes.len());

	for volume in &volumes {
		println!("Volume: {}", volume.mount_point.display());
		println!("  Name: {} (stable: UUID in name)", volume.name);
		println!("  Filesystem: {} (stable)", volume.file_system);
		println!(
			"  Total capacity: {} bytes (stable: physical drive size)",
			volume.total_capacity
		);
		println!(
			"  Available: {} bytes (CHANGES: as files are added/deleted)",
			volume.available_space
		);

		if let Some(container) = &volume.apfs_container {
			println!("\n  APFS Container:");
			println!(
				"    container_id: {} (CHANGES: disk3 → disk4 on reboot)",
				container.container_id
			);
			println!("    uuid: {} (STABLE: always same)", container.uuid);
			println!(
				"    total_capacity: {} bytes (STABLE)",
				container.total_capacity
			);

			// Check individual volume properties
			for vol in &container.volumes {
				println!("\n    Volume {} in container:", vol.name);
				println!(
					"      disk_id: {} (CHANGES: disk3s5 → disk4s5 on reboot)",
					vol.disk_id
				);
				println!("      uuid: {} (STABLE)", vol.uuid);
				println!(
					"      capacity_consumed: {} bytes (CHANGES: with file operations)",
					vol.capacity_consumed
				);
			}
		}

		println!("\n  For stable fingerprint, should use:");
		println!("     container.uuid:volume.uuid");
		println!("     container.total_capacity (physical drive size)");
		println!("     NOT container_id (changes on reboot)");
		println!("     NOT capacity_consumed (changes with files)");
		println!();
	}
}
