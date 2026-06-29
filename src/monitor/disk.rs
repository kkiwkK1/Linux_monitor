use super::DiskMetrics;
use std::ffi::CString;
use std::fs;
use std::mem;

pub fn collect_disk() -> Vec<DiskMetrics> {
    let mut metrics = Vec::new();

    let mounts = match fs::read_to_string("/proc/mounts") {
        Ok(c) => c,
        Err(_) => return metrics,
    };

    for line in mounts.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 {
            continue;
        }
        let device = parts[0];
        let mount_point = parts[1];
        let fs_type = parts[2];

        // Only track real filesystems
        let is_real = matches!(
            fs_type,
            "ext2" | "ext3" | "ext4" | "xfs" | "btrfs" | "zfs" | "ntfs" | "vfat" | "fuse"
        );
        if !is_real {
            continue;
        }
        if device.starts_with("overlay") || device.starts_with("tmpfs") || device.starts_with("devtmpfs") {
            continue;
        }

        // Use libc statvfs for accurate disk space
        let mount_cstr = match CString::new(mount_point) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let mut stat: libc::statvfs = unsafe { mem::zeroed() };
        let rc = unsafe { libc::statvfs(mount_cstr.as_ptr(), &mut stat) };
        if rc != 0 {
            continue;
        }

        let block_size = stat.f_frsize.max(stat.f_bsize) as u64;
        let total_kb = (stat.f_blocks as u64 * block_size) / 1024;
        let available_kb = (stat.f_bavail as u64 * block_size) / 1024;
        let used_kb = total_kb.saturating_sub(available_kb);
        let usage_percent = if total_kb > 0 {
            (used_kb as f32 / total_kb as f32) * 100.0
        } else {
            0.0
        };

        metrics.push(DiskMetrics {
            mount_point: mount_point.to_string(),
            device: device.to_string(),
            total_kb,
            used_kb,
            available_kb,
            usage_percent,
            read_bytes: 0,
            write_bytes: 0,
        });
    }

    metrics
}
