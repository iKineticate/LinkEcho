use winsafe::{HrResult, SHGetKnownFolderPath, co};

#[derive(Default)]
pub enum SystemLinkDirs {
    #[default]
    Desktop,
    StartMenu,
    StartUp,
}

impl SystemLinkDirs {
    pub fn get_path(&self) -> HrResult<Vec<String>> {
        let know_folder_id_vec = match self {
            SystemLinkDirs::Desktop => {
                vec![co::KNOWNFOLDERID::Desktop, co::KNOWNFOLDERID::PublicDesktop]
            }
            SystemLinkDirs::StartMenu => vec![
                co::KNOWNFOLDERID::StartMenu,
                co::KNOWNFOLDERID::CommonStartMenu,
            ],
            SystemLinkDirs::StartUp => {
                vec![co::KNOWNFOLDERID::Startup, co::KNOWNFOLDERID::CommonStartup]
            }
        };

        know_folder_id_vec
            .into_iter()
            .map(|id| SHGetKnownFolderPath(&id, co::KF::NO_ALIAS, None))
            .collect::<HrResult<Vec<_>>>()
    }
}
