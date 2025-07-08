// ref: https://gitlab.freedesktop.org/mesa/mesa/-/blob/main/src/util/u_process.c

use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ProcProgEntry {
    pub pid: i32,
    pub name: String,
}

impl ProcProgEntry {
    pub fn get_from_pid(pid: i32) -> Option<Self> {
        let path = PathBuf::from(format!("/proc/{pid}"));
        let name = get_name_from_proc_path(path)?;

        Some(Self { pid, name })
    }

    pub fn get_self() -> Option<Self> {
        let path = fs::read_link("/proc/self").ok()?;
        let pid = get_pid_from_proc_path(&path)?;
        let name = get_name_from_proc_path(&path)?;

        Some(Self { pid, name })
    }

    pub fn get_all_proc_prog_entries() -> Vec<Self> {
        let mut buf = Vec::with_capacity(64);

        Self::update_entries(&mut buf);

        buf
    }

    pub fn update_entries(buf: &mut Vec<Self>) {
        let Ok(proc_dir) = fs::read_dir("/proc") else {
            return;
        };

        for dir_entry in proc_dir {
            let Ok(dir_entry) = dir_entry else { continue };
            let path = dir_entry.path();

            let Some(name) = get_name_from_proc_path(&path) else {
                continue;
            };
            let Some(s) = path.file_name().and_then(|file_name| file_name.to_str()) else {
                continue;
            };
            let Ok(pid) = s.parse::<i32>() else { continue };

            buf.push(Self { name, pid });
        }
    }

    pub fn update_entries_with_name_filter<T: AsRef<str>>(buf: &mut Vec<Self>, name_filter: &[T]) {
        let Ok(proc_dir) = fs::read_dir("/proc") else {
            return;
        };

        for dir_entry in proc_dir {
            let Ok(dir_entry) = dir_entry else { continue };
            let path = dir_entry.path();

            let Some(name) = get_name_from_proc_path(&path) else {
                continue;
            };

            if !name_filter
                .iter()
                .map(|s| s.as_ref())
                .any(|filter| filter == name)
            {
                continue;
            }

            let Some(s) = path.file_name().and_then(|file_name| file_name.to_str()) else {
                continue;
            };
            let Ok(pid) = s.parse::<i32>() else { continue };

            buf.push(Self { name, pid });
        }
    }

    pub fn get_all_entries_with_name_filter<T: AsRef<str>>(name_filter: &[T]) -> Vec<Self> {
        let mut buf = Vec::with_capacity(64);

        Self::update_entries_with_name_filter(&mut buf, name_filter);

        buf
    }
}

// `/proc/<pid>`
fn get_pid_from_proc_path<P: AsRef<Path>>(path: P) -> Option<i32> {
    path.as_ref()
        .file_name()
        .and_then(|name| name.to_str())?
        .parse::<i32>()
        .ok()
}

// `/proc/<pid>`
fn get_name_from_proc_path<P: AsRef<Path>>(path: P) -> Option<String> {
    let path = path.as_ref();
    let exe = fs::read_link(path.join("exe")).ok()?.display().to_string();
    let comm = fs::read_to_string(path.join("comm")).ok()?;
    let name_from_exe = get_name_from_str(&exe)?;

    let name = if name_from_exe.starts_with(comm.trim_end()) {
        name_from_exe
    } else {
        let cmdline = {
            let mut buf_cmdline = [0u8; 128];
            let mut f = File::open(path.join("cmdline")).ok()?;
            f.read_exact(&mut buf_cmdline);
            String::from_utf8_lossy(&buf_cmdline).into_owned()
        };
        let name_from_cmdline = get_name_from_str(&cmdline)?;

        name_from_cmdline
    };

    Some(name)
}

fn get_name_from_str(s: &str) -> Option<String> {
    let s = if let Some((ss, _)) = s.split_once('\0') {
        ss
    } else {
        s
    };
    let s = s.split(['\\', '/']).last()?;

    if s.is_empty() {
        return None;
    }

    Some(s.to_string())
}
