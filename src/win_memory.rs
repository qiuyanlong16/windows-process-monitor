use std::mem::size_of;
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS_EX2};
use windows::Win32::System::Threading::{
    OpenProcess, OpenProcessToken, PROCESS_QUERY_INFORMATION, PROCESS_QUERY_LIMITED_INFORMATION,
    PROCESS_VM_READ,
};
use windows::Win32::Security::{GetTokenInformation, TOKEN_MANDATORY_LABEL, TOKEN_QUERY};

#[repr(C)]
struct RawSid {
    revision: u8,
    sub_authority_count: u8,
    authority: [u8; 6],
}

#[repr(C)]
struct RawSidWithSubAuthorities {
    revision: u8,
    sub_authority_count: u8,
    authority: [u8; 6],
    sub_authorities: [u32; 1],
}

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

/// TokenIntegrityLevel = 25 (from winnt.h)
const TOKEN_INTEGRITY_LEVEL: u32 = 25;

pub fn get_process_privilege(pid: u32) -> Option<&'static str> {
    if pid == 0 {
        return None;
    }

    unsafe {
        let handle = open_process(pid)?;
        let mut token = HANDLE::default();
        if OpenProcessToken(handle, TOKEN_QUERY, &mut token).is_err() {
            let _ = CloseHandle(handle);
            return None;
        }

        let mut size_needed: u32 = 0;
        let _ = GetTokenInformation(
            token,
            std::mem::transmute::<u32, windows::Win32::Security::TOKEN_INFORMATION_CLASS>(TOKEN_INTEGRITY_LEVEL),
            None,
            0,
            &mut size_needed,
        );

        if size_needed == 0 {
            let _ = CloseHandle(token);
            let _ = CloseHandle(handle);
            return None;
        }

        let mut buffer: Vec<u8> = vec![0; size_needed as usize];
        let result = GetTokenInformation(
            token,
            std::mem::transmute::<u32, windows::Win32::Security::TOKEN_INFORMATION_CLASS>(TOKEN_INTEGRITY_LEVEL),
            Some(buffer.as_mut_ptr() as *mut _),
            size_needed,
            &mut size_needed,
        );
        let _ = CloseHandle(token);
        let _ = CloseHandle(handle);
        result.ok()?;

        let label = &*(buffer.as_ptr() as *const TOKEN_MANDATORY_LABEL);
        let sid_ptr = label.Label.Sid.0;
        if sid_ptr.is_null() {
            return None;
        }

        let raw_sid = &*(sid_ptr as *const RawSid);
        let sub_auth_count = raw_sid.sub_authority_count as u32;
        if sub_auth_count == 0 {
            return None;
        }

        let last_idx = sub_auth_count - 1;
        let sid_with_auth = sid_ptr as *const RawSidWithSubAuthorities;
        let rid = (*sid_with_auth).sub_authorities[last_idx as usize];

        Some(integrity_label(rid))
    }
}

fn integrity_label(rid: u32) -> &'static str {
    match rid {
        0x0000 => "Untrusted",
        0x1000 => "Low",
        0x2000..=0x20FF => "Medium",
        0x2100..=0x2FFF => "Med-Hi",
        0x3000..=0x3FFF => "Admin",
        0x4000..=0x4FFF => "System",
        _ => "Protected",
    }
}
