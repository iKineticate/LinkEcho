use crate::AppState;

use windows_reactor::*;

pub enum ContentDialogType {
    ChangeAllShortcutIcon,
    ChangeSpecifiedShortcutIcon(/* Name */ String),
    RestoreAllShortcutIcon,
    RestoreSpecifiedShortcutIcon(/* Name */ String),
}

// pub fn content_dialog(app_state: AppState) -> Element { 


// }