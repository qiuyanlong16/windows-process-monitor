use std::mem::size_of;
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS_EX2};
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_VM_READ,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProcessMemoryUsage {
    pub working_set_bytes: u64,
    pub private_working_set_bytes: u64,
}

pub fn process_memory_usage(pid: u32) -> Option<ProcessMemoryUsage> {
    if pid == 0 {
        return None;
    }

    unsafe {
        let handle = open_process(pid)?;
        let memory = query_memory_usage(handle);
        let _ = CloseHandle(handle);
        memory
    }
}

unsafe fn open_process(pid: u32) -> Option<HANDLE> {
    match OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, pid) {
        Ok(handle) => Some(handle),
        Err(_) => OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok(),
    }
}

unsafe fn query_memory_usage(handle: HANDLE) -> Option<ProcessMemoryUsage> {
    let mut counters = PROCESS_MEMORY_COUNTERS_EX2::default();
    counters.cb = size_of::<PROCESS_MEMORY_COUNTERS_EX2>() as u32;

    GetProcessMemoryInfo(
        handle,
        &mut counters as *mut _ as *mut _,
        counters.cb,
    )
    .ok()?;

    Some(ProcessMemoryUsage {
        working_set_bytes: counters.WorkingSetSize as u64,
        private_working_set_bytes: counters.PrivateWorkingSetSize as u64,
    })
}
