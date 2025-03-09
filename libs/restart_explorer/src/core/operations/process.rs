use std::ffi::CStr;

use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::CloseHandle,
        System::{
            Diagnostics::ToolHelp::{
                CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32,
                TH32CS_SNAPPROCESS,
            },
            Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE},
        },
        UI::{Shell::ShellExecuteW, WindowsAndMessaging::SW_NORMAL},
    },
};

pub fn start_process(process_name: &str) {
    let process_name_utf16: Vec<u16> = process_name
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    unsafe {
        ShellExecuteW(
            None,
            None,
            PCWSTR(process_name_utf16.as_ptr()),
            None,
            None,
            SW_NORMAL,
        )
    };
}

fn kill_process(process_id: u32) {
    unsafe {
        if let Ok(process) = OpenProcess(PROCESS_TERMINATE, false, process_id) {
            let _ = TerminateProcess(process, 1);
            let _ = CloseHandle(process);
        }
    }
}

pub fn kill_process_by_name(process_name: &str) {
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).unwrap() };
    let mut entry = PROCESSENTRY32::default();
    entry.dwSize = std::mem::size_of::<PROCESSENTRY32>() as u32;
    let mut has_next_process = unsafe { Process32First(snapshot, &mut entry).is_ok() };
    while has_next_process {
        let current_process_name =
            unsafe { CStr::from_ptr(entry.szExeFile.as_ptr()).to_str().unwrap() };
        if current_process_name == process_name {
            kill_process(entry.th32ProcessID);
        }

        has_next_process = unsafe { Process32Next(snapshot, &mut entry).is_ok() };
    }

    unsafe {
        let _ = CloseHandle(snapshot);
    };
}
