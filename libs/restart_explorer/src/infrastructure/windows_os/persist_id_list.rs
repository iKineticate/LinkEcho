use windows::Win32::UI::Shell::{Common, IPersistIDList, IShellItem, SHCreateItemFromIDList};

use super::shell_item::ShellItem;

pub trait PersistIDList {
    fn get_id_list(&self) -> windows_core::Result<*mut Common::ITEMIDLIST>;
    fn id_list_into_shell_item(
        &self,
        id_list: *mut Common::ITEMIDLIST,
    ) -> windows_core::Result<impl ShellItem>;
}

impl PersistIDList for IPersistIDList {
    fn get_id_list(&self) -> windows_core::Result<*mut Common::ITEMIDLIST> {
        unsafe { self.GetIDList() }
    }

    fn id_list_into_shell_item(
        &self,
        id_list: *mut Common::ITEMIDLIST,
    ) -> windows_core::Result<impl ShellItem> {
        unsafe { SHCreateItemFromIDList::<IShellItem>(id_list) }
    }
}
