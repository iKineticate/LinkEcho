use std::time::{Duration, Instant};

use windows::{
    core::w,
    Win32::{
        Foundation::{ERROR_TIMEOUT, WIN32_ERROR},
        UI::WindowsAndMessaging::{FindWindowW, IsWindowVisible},
    },
};

pub fn wait_for_explorer_stable(timeout: Duration) -> Result<(), windows::core::Error> {
    let start = Instant::now();

    loop {
        if start.elapsed() > timeout {
            return Err(windows::core::Error::from(WIN32_ERROR(ERROR_TIMEOUT.0)));
        }

        unsafe {
            let progman_window = FindWindowW(w!("Progman"), None);
            if progman_window.is_err() || !IsWindowVisible(progman_window.unwrap()).as_bool() {
                std::thread::sleep(Duration::from_millis(100));
                continue;
            }

            let explorer_window = FindWindowW(w!("Shell_TrayWnd"), None);
            if explorer_window.is_err() || !IsWindowVisible(explorer_window.unwrap()).as_bool() {
                std::thread::sleep(Duration::from_millis(100));
                continue;
            }
        }

        break;
    }

    Ok(())
}
