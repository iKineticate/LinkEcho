mod env_path;
pub mod info;
pub mod shell_folders;

use getset::{Getters, Setters};
use windows_reactor::SurfaceImageSource;

#[derive(Debug, Clone, PartialEq)]
pub enum IconResourceType {
    Dll(String, Option<i32>),
    Exe(String),
    Icon(String),
    Other(String),
}

impl Default for IconResourceType {
    fn default() -> Self {
        IconResourceType::Icon(String::new())
    }
}

impl From<String> for IconResourceType {
    fn from(path: String) -> Self {
        infer::get_from_path(&path)
            .ok()
            .flatten()
            .map_or(Self::Other(path.to_owned()), |t| match t.extension() {
                "ico" => Self::Icon(path),
                "dll" => Self::Dll(path, None),
                "exe" => Self::Exe(path),
                _ => Self::Other(path),
            })
    }
}

impl AsRef<str> for IconResourceType {
    fn as_ref(&self) -> &str {
        match self {
            Self::Icon(path) | Self::Exe(path) | Self::Dll(path, _) | Self::Other(path) => {
                path.as_str()
            }
        }
    }
}

impl IconResourceType {
    pub fn set_dll_index(&mut self, index: i32) -> Result<(), &str> {
        match self {
            Self::Dll(_path, icon_index) => {
                *icon_index = Some(index);
                Ok(())
            }
            _ => Err("the icon resource is not dll"),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Setters, Getters)]
#[getset(set = "pub", get = "pub")]
pub struct LinkInfo {
    name: String,
    path: String,
    icon_path: Option<IconResourceType>,
    icon_surface: Option<SurfaceImageSource>,
    target_path: String,
    target_icon_surface: Option<SurfaceImageSource>,
    working_directory: String,
    status: bool,
}
