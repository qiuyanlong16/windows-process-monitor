use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use sysinfo::{ProcessesToUpdate, ProcessRefreshKind, RefreshKind, System};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessStats {
    pub name: String,
    pub pid: u32,
    pub cpu_percent: f32,
    pub memory_bytes: u64,
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
        std::thread::sleep(std::time::Duration::from_millis(500));

        loop {
            sys.refresh_cpu_usage();
            sys.refresh_memory();

            if watch_process.is_some() {
                sys.refresh_processes_specifics(
                    ProcessesToUpdate::All,
                    false,
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

fn find_process(sys: &System, name: &str) -> Option<ProcessStats> {
    let name_lower = name.to_lowercase();
    sys.processes()
        .values()
        .filter(|p| {
            p.name()
                .to_string_lossy()
                .to_lowercase()
                == name_lower
        })
        .max_by_key(|p| p.memory())
        .map(|p| ProcessStats {
            name: name.to_string(),
            pid: p.pid().as_u32(),
            cpu_percent: p.cpu_usage(),
            memory_bytes: p.memory(),
        })
}
