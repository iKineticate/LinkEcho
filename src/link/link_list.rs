use crate::link::link_info::{ManageLinkProp, SystemLinkDirs};
use crate::t;
use std::path::PathBuf;

pub struct ListState {
    pub select: Option<usize>,
}

impl Default for ListState {
    fn default() -> Self {
        Self { select: None }
    }
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
            .expect("Failed to get desktops path");
        let link_vec = ManageLinkProp::collect(&desktop_path)
            .expect("Failed to get properties of desktop shortcuts");

        Self {
            items: link_vec,
            state: ListState::default(),
            source: ShortcutSource::Desktop,
        }
    }

    pub fn start_menu() -> Self {
        let start_path = SystemLinkDirs::StartMenu
            .get_path()
            .expect("Failed to get Start Menu path");
        let link_vec = ManageLinkProp::collect(&start_path)
            .expect("Failed to get properties of start menu shortcuts");

        Self {
            items: link_vec,
            state: ListState::default(),
            source: ShortcutSource::StartMenu,
        }
    }

    pub fn other(path: PathBuf) -> Self {
        let link_vec = ManageLinkProp::collect(&vec![path.clone()])
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

#[derive(Clone, PartialEq)]
pub struct LinkProp {
    pub name: String,
    pub path: String,
    pub status: Status,
    pub target_ext: String,
    pub target_dir: String,
    pub target_path: String,
    pub icon: String,
    pub target_icon: String,
    pub icon_location: String,
    pub icon_index: String,
    pub arguments: String,
    pub file_size: String,
    pub created_at: String,
    pub updated_at: String,
    pub accessed_at: String,
}
