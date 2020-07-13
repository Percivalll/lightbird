use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use libc::statvfs;
use std::mem;
use std::ffi::CString;
use serde::{Deserialize, Serialize};
#[derive(Debug,Serialize, Deserialize)]
pub struct Disk {
    name: String,
    file_system: String,
    mount_point: String,
    total_space: u64,
    available_space: u64,
    genre: String,
}
fn find_type_for_name(name: &str) -> String {
    let name_path = name;
    let real_path = fs::canonicalize(name_path).unwrap_or_else(|_| PathBuf::from(name_path));
    let mut real_path = real_path.to_str().unwrap_or_default();
    if name_path.starts_with("/dev/mapper/") {
        // Recursively solve, for example /dev/dm-0
        if real_path != name_path {
            return find_type_for_name(real_path);
        }
    } else if name_path.starts_with("/dev/sd") {
        // Turn "sda1" into "sda"
        real_path = real_path.trim_start_matches("/dev/");
        real_path = real_path.trim_end_matches(|c| c >= '0' && c <= '9');
    } else if name_path.starts_with("/dev/nvme") {
        // Turn "nvme0n1p1" into "nvme0n1"
        real_path = real_path.trim_start_matches("/dev/");
        real_path = real_path.trim_end_matches(|c| c >= '0' && c <= '9');
        real_path = real_path.trim_end_matches(|c| c == 'p');
    } else if name_path.starts_with("/dev/root") {
        // Recursively solve, for example /dev/mmcblk0p1
        if real_path != name_path {
            return find_type_for_name(real_path);
        }
    } else if name_path.starts_with("/dev/mmcblk") {
        // Turn "mmcblk0p1" into "mmcblk0"
        real_path = real_path.trim_start_matches("/dev/");
        real_path = real_path.trim_end_matches(|c| c >= '0' && c <= '9');
        real_path = real_path.trim_end_matches(|c| c == 'p');
    } else {
        // Default case: remove /dev/ and expects the name presents under /sys/block/
        // For example, /dev/dm-0 to dm-0
        real_path = real_path.trim_start_matches("/dev/");
    }

    let trimmed: &OsStr = OsStrExt::from_bytes(real_path.as_bytes());
    let path = Path::new("/sys/block/")
        .to_owned()
        .join(trimmed)
        .join("queue/rotational");
    // Normally, this file only contains '0' or '1' but just in case, we get 8 bytes...
    let mut content = String::new();
    match File::open(path) {
        Ok(o) => o,
        Err(err) => return String::from("Unknown"),
    }
    .read_to_string(&mut content)
    .unwrap();
    match content.as_str().trim() {
        "1" => return String::from("HDD"),
        "0" => return String::from("SDD"),
        _ => return String::from("Unknown"),
    }
}
pub fn get_disk() -> Vec<Disk> {
    let mut file = File::open("/proc/mounts").unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content);
    content
        .lines()
        .map(|line| {
            let line = line.trim();
            let mut fields = line.split_whitespace();
            let fs_spec = fields.next().unwrap_or("");
            let fs_file = fields.next().unwrap_or("");
            let fs_vfstype = fields.next().unwrap_or("");
            (fs_spec, fs_file, fs_vfstype)
        })
        .filter(|(fs_spec, fs_file, fs_vfstype)| {
            let filtered = match *fs_vfstype {
            "devpts"|
            "mqueue"|
            "hugetlbfs"|
            "sysfs" | // pseudo file system for kernel objects
            "proc" |  // another pseudo file system
            "tmpfs" |
            "devtmpfs" |
            "cgroup" |
            "cgroup2" |
            "pstore" | // https://www.kernel.org/doc/Documentation/ABI/testing/pstore
            "squashfs" | // squashfs is a compressed read-only file system (for snaps)
            "rpc_pipefs" | // The pipefs pseudo file system service
            "iso9660" => true, // optical media
            _ => false,
        };
            !(filtered ||
            fs_file.starts_with("/sys") || // check if fs_file is an 'ignored' mount point
            fs_file.starts_with("/proc") ||
            fs_file.starts_with("/run") ||
            fs_spec.starts_with("sunrpc"))
        })
        .map(|(fs_spec, fs_file, fs_vfstype)| {
            let mut total=0;
            let mut available=0;
            unsafe {
                let mut stat: statvfs = mem::zeroed();
                if statvfs(CString::new("/").unwrap().as_ptr(), &mut stat) == 0 {
                    total = u64::from(stat.f_bsize) * u64::from(stat.f_blocks);
                    available = u64::from(stat.f_bsize) * u64::from(stat.f_bavail);
                }
            }
            Disk {
            name: fs_spec.to_string(),
            mount_point: fs_file.to_string(),
            file_system: fs_vfstype.to_string(),
            total_space: total,
            available_space: available,
            genre: find_type_for_name(fs_spec),
        }})
        .collect()
}
#[test]
fn get_disk_test() {
    for i in get_disk() {
        println!("{:?}", i);
    }
}
