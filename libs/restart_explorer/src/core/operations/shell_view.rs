use windows::Win32::System::Com::CoTaskMemFree;

use crate::infrastructure::windows_os::{
    persist_id_list::PersistIDList, shell_item::ShellItem, shell_view::ShellView,
};

pub fn get_path_from_shell_view<TShellView: ShellView>(
    shell_view: &TShellView,
) -> Result<String, windows::core::Error> {
    let persist_id_list = shell_view.as_persist_id_list()?;
    let id_list = persist_id_list.get_id_list()?;
    let item = persist_id_list.id_list_into_shell_item(id_list)?;
    let path = item.get_display_name()?;
    unsafe { CoTaskMemFree(Option::Some(id_list as _)) };
    Ok(path)
}
