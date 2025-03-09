use windows::Win32::{
    Foundation::HWND,
    System::Com::{CoCreateInstance, CLSCTX_LOCAL_SERVER},
    UI::{
        Shell::IShellWindows,
        WindowsAndMessaging::{GetParent, GetTopWindow, GetWindow, GET_WINDOW_CMD},
    },
};

use super::shell_windows::ShellWindows;

pub trait WindowApi {
    fn get_top_window(&self, hwnd: HWND) -> windows::core::Result<HWND>;
    fn get_window(&self, hwnd: HWND, command: GET_WINDOW_CMD) -> windows::core::Result<HWND>;
    fn get_parent(&self, hwnd: HWND) -> windows::core::Result<HWND>;
    fn create_shell_windows(&self) -> windows::core::Result<impl ShellWindows>;
}

pub struct Win32WindowApi;

impl WindowApi for Win32WindowApi {
    fn get_top_window(&self, hwnd: HWND) -> windows::core::Result<HWND> {
        unsafe { Ok(GetTopWindow(hwnd)?) }
    }

    fn get_window(&self, hwnd: HWND, command: GET_WINDOW_CMD) -> windows::core::Result<HWND> {
        unsafe { Ok(GetWindow(hwnd, command)?) }
    }

    fn get_parent(&self, hwnd: HWND) -> windows::core::Result<HWND> {
        unsafe { Ok(GetParent(hwnd)?) }
    }

    fn create_shell_windows(&self) -> windows::core::Result<impl ShellWindows> {
        let shell_windows: IShellWindows = unsafe {
            CoCreateInstance(
                &windows::Win32::UI::Shell::ShellWindows,
                None,
                CLSCTX_LOCAL_SERVER,
            )
        }?;
        Ok(shell_windows)
    }
}
