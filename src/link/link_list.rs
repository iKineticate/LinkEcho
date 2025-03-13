use crate::link::link_info::{ManageLinkProp, SystemLinkDirs};
use crate::t;
use log::*;
use std::path::PathBuf;

#[derive(Default)]
pub struct ListState {
    pub select: Option<usize>,
}

pub enum ShortcutSource {
    Desktop,
    StartMenu,
    Other(PathBuf),
}

impl ShortcutSource {
    pub fn name(&self) -> String {
        match self {
            ShortcutSource::Desktop => t!("DESKTOP").into_owned(),
            ShortcutSource::StartMenu => t!("START_MENU").into_owned(),
            ShortcutSource::Other(path) => path
                .file_name()
                .map_or("None".to_owned(), |n| n.to_string_lossy().into_owned()),
        }
    }
}

pub struct LinkList {
    pub items: Vec<LinkProp>,
    pub state: ListState,
    pub source: ShortcutSource,
}

impl Default for LinkList {
    fn default() -> Self {
        LinkList::desktop()
    }
}

impl LinkList {
    pub fn desktop() -> Self {
        let desktop_path = SystemLinkDirs::Desktop
            .get_path()
            .map_err(|e| error!("Failed to get Desktops path: {e}"))
            .expect("Failed to get Desktops path");
        let link_vec = ManageLinkProp::collect(&desktop_path)
            .map_err(|e| error!("Failed to get properties of Desktop shortcuts: {e}"))
            .expect("Failed to get properties of Desktop shortcuts");

        Self {
            items: link_vec,
            state: ListState::default(),
            source: ShortcutSource::Desktop,
        }
    }

    pub fn start_menu() -> Self {
        let start_path = SystemLinkDirs::StartMenu
            .get_path()
            .map_err(|e| error!("Failed to get Start Menu path: {e}"))
            .expect("Failed to get Start Menu path");
        let link_vec = ManageLinkProp::collect(&start_path)
            .map_err(|e| error!("Failed to get properties of Start Menu shortcuts: {e}"))
            .expect("Failed to get properties of Start Menu shortcuts");

        Self {
            items: link_vec,
            state: ListState::default(),
            source: ShortcutSource::StartMenu,
        }
    }

    pub fn other(path: PathBuf) -> Self {
        let link_vec = ManageLinkProp::collect(&[path.clone()])
            .map_err(|e| error!("Failed to get properties of desktop shortcuts: {e}"))
            .expect("Failed to get properties of desktop shortcuts");

        Self {
            items: link_vec,
            state: ListState::default(),
            source: ShortcutSource::Other(path),
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum Status {
    Unchanged,
    Changed,
}

impl Default for Status {
    fn default() -> Self {
        Status::Unchanged
    }
}

#[derive(Default, Clone, PartialEq)]
pub struct LinkProp {
    pub name: String,
    pub path: String,
    pub status: Status,
    pub target_ext: String,
    pub target_dir: String,
    pub target_path: String,
    pub icon_base64: String,
    pub target_icon_base64: String,
    pub icon_path: String,
    pub icon_index: String,
    pub arguments: String,
    pub file_size: String,
    pub created_at: String,
    pub updated_at: String,
    pub accessed_at: String,
}
