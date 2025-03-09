use windows::Win32::{
    Foundation::HWND,
    UI::Shell::{IPersistIDList, IShellView},
};
use windows_core::Interface;

use super::persist_id_list::PersistIDList;

pub trait ShellView {
    fn get_window(&self) -> windows::core::Result<HWND>;
    fn as_persist_id_list(&self) -> windows::core::Result<impl PersistIDList>;
}

impl ShellView for IShellView {
    fn get_window(&self) -> windows::core::Result<HWND> {
        unsafe { self.GetWindow() }
    }

    fn as_persist_id_list(&self) -> windows::core::Result<impl PersistIDList> {
        Interface::cast::<IPersistIDList>(self)
    }
}
