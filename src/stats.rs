use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use sysinfo::{Process, ProcessesToUpdate, ProcessRefreshKind, RefreshKind, System, MINIMUM_CPU_UPDATE_INTERVAL};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessStats {
    pub name: String,
    pub pid: u32,
    pub cpu_percent: f32,
    pub working_set_bytes: u64,
    pub private_working_set_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stats {
    pub cpu_percent: f32,
    pub mem_used: u64,
    pub mem_total: u64,
    pub watched_process: Option<ProcessStats>,
}

impl Stats {
    pub fn new() -> Self {
        Stats {
            cpu_percent: 0.0,
            mem_used: 0,
            mem_total: 0,
            watched_process: None,
        }
    }
}

pub type SharedStats = Arc<RwLock<Stats>>;

pub fn create_shared_stats() -> SharedStats {
    Arc::new(RwLock::new(Stats::new()))
}

pub fn spawn_stats_collector(shared: SharedStats, watch_process: Option<String>) {
    std::thread::spawn(move || {
        let mut sys = System::new_with_specifics(
            RefreshKind::nothing()
                .with_memory(sysinfo::MemoryRefreshKind::nothing().with_ram())
                .with_cpu(sysinfo::CpuRefreshKind::nothing().with_cpu_usage()),
        );

        // First refresh to seed data (CPU usage needs a delta measurement)
        sys.refresh_cpu_usage();
        sys.refresh_memory();
        if watch_process.is_some() {
            sys.refresh_processes_specifics(
                ProcessesToUpdate::All,
                true,
                ProcessRefreshKind::nothing().with_cpu().with_memory(),
            );
        }
        std::thread::sleep(MINIMUM_CPU_UPDATE_INTERVAL);

        loop {
            sys.refresh_cpu_usage();
            sys.refresh_memory();

            if watch_process.is_some() {
                sys.refresh_processes_specifics(
                    ProcessesToUpdate::All,
                    true,
                    ProcessRefreshKind::nothing().with_cpu().with_memory(),
                );
            }

            let mut stats = shared.write().unwrap();
            stats.cpu_percent = sys.global_cpu_usage();
            stats.mem_used = sys.used_memory();
            stats.mem_total = sys.total_memory();

            if let Some(ref proc_name) = watch_process {
                stats.watched_process = find_process(&sys, proc_name);
            }

            drop(stats);
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    });
}

fn normalize_process_name(name: &str) -> String {
    let lower = name.to_lowercase();
    lower.strip_suffix(".exe").unwrap_or(&lower).to_string()
}

fn find_process(sys: &System, name: &str) -> Option<ProcessStats> {
    let watch_base = normalize_process_name(name);
    let matching: Vec<(&Process, ProcessMemoryUsage)> = sys
        .processes()
        .values()
        .filter(|p| normalize_process_name(&p.name().to_string_lossy()) == watch_base)
        .filter_map(|p| process_memory_usage(p).map(|usage| (p, usage)))
        .collect();

    if matching.is_empty() {
        return None;
    }

    let cpu_percent: f32 = matching.iter().map(|(p, _)| p.cpu_usage()).sum();
    let (working_set_bytes, private_working_set_bytes) = matching
        .iter()
        .map(|(_, usage)| *usage)
        .fold((0u64, 0u64), |(ws, pws), usage| {
            (ws + usage.working_set_bytes, pws + usage.private_working_set_bytes)
        });
    let pid = matching
        .iter()
        .max_by_key(|(_, usage)| usage.working_set_bytes)
        .map(|(p, _)| p.pid().as_u32())
        .unwrap_or(0);
    let display_name = if matching.len() > 1 {
        format!("{} ({})", watch_base, matching.len())
    } else {
        name.to_string()
    };

    Some(ProcessStats {
        name: display_name,
        pid,
        cpu_percent,
        working_set_bytes,
        private_working_set_bytes,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ProcessMemoryUsage {
    working_set_bytes: u64,
    private_working_set_bytes: u64,
}

fn process_memory_usage(process: &Process) -> Option<ProcessMemoryUsage> {
    #[cfg(windows)]
    {
        return crate::win_memory::process_memory_usage(process.pid().as_u32()).map(|usage| {
            ProcessMemoryUsage {
                working_set_bytes: usage.working_set_bytes,
                private_working_set_bytes: usage.private_working_set_bytes,
            }
        });
    }

    #[cfg(not(windows))]
    {
        let bytes = process.memory();
        Some(ProcessMemoryUsage {
            working_set_bytes: bytes,
            private_working_set_bytes: bytes,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::normalize_process_name;

    #[test]
    fn normalize_process_name_strips_exe_and_lowercases() {
        assert_eq!(normalize_process_name("by-claw.exe"), "by-claw");
        assert_eq!(normalize_process_name("BY-CLAW"), "by-claw");
        assert_eq!(normalize_process_name("chrome.EXE"), "chrome");
    }
}
