use std::fs;
use std::path::{Path,PathBuf};

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
        let Ok(proc_dir) = fs::read_dir("/proc") else { return Vec::new() };

        proc_dir.filter_map(|dir_entry| {
            let path = dir_entry.ok()?.path();
            let name = get_name_from_proc_path(&path)?;
            let pid = path.file_name().and_then(|name| name.to_str())?.parse::<i32>().ok()?;

            Some(Self { pid, name })
        }).collect()
    }

    pub fn get_all_entries_with_name_filter(name_filter: &[&str]) -> Vec<Self> {
        let Ok(proc_dir) = fs::read_dir("/proc") else { return Vec::new() };

        proc_dir.filter_map(|dir_entry| {
            let path = dir_entry.ok()?.path();
            let name = get_name_from_proc_path(&path)?;

            if !name_filter.contains(&name.as_str()) {
                return None;
            }

            let pid = path.file_name().and_then(|name| name.to_str())?.parse::<i32>().ok()?;

            Some(Self { pid, name })
        }).collect()
    }
}

// `/proc/<pid>`
fn get_pid_from_proc_path<P: AsRef<Path>>(path: P) -> Option<i32> {
    path.as_ref().file_name().and_then(|name| name.to_str())?.parse::<i32>().ok()
}

// `/proc/<pid>`
fn get_name_from_proc_path<P: AsRef<Path>>(path: P) -> Option<String> {
    let path = path.as_ref();
    let exe = fs::read_link(path.join("exe")).ok()?.display().to_string();
    let cmdline = fs::read_to_string(path.join("cmdline")).ok()?;
    let comm = fs::read_to_string(path.join("comm")).ok()?;

    let [name_from_exe, name_from_cmdline] = [exe, cmdline].map(|s| get_name_from_str(&s));
    let [name_from_exe, name_from_cmdline] = [name_from_exe?, name_from_cmdline?];

    let name = if name_from_cmdline.starts_with(comm.trim_end()) {
        name_from_cmdline
    } else {
        name_from_exe
    };

    Some(name)
}

fn get_name_from_str(s: &str) -> Option<String> {
    let s = if let Some((ss, _)) = s.split_once('\0') {
        ss
    } else {
        &s
    };
    let s = s.split(['\\', '/']).last()?;

    if s.is_empty() { return None }

    Some(s.to_string())
}
