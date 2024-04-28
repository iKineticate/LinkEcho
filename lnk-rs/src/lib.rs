#![warn(missing_docs)]

//! # Shell Link parser and writer for Rust.
//!
//! Works on any OS - although only really useful in Windows, this library can parse and write
//! .lnk files, a shell link, that can be understood by Windows.
//!
//! To get started, see the [ShellLink](struct.ShellLink.html) struct.
//!
//! The full specification of these files can be found at
//! [Microsoft's Website](https://docs.microsoft.com/en-us/openspecs/windows_protocols/ms-shllink/16cb4ca1-9339-4d0c-a68d-bf1d6cc0f943).
//!
//! ## Read Example
//!
//! A simple example appears as follows:
//! ```
//! use lnk::ShellLink;
//! // ...
//! let shortcut = lnk::ShellLink::open("tests/test.lnk").unwrap();
//! println!("{:#?}", shortcut);
//! ```
//!
//! ## Write Example
//!
//! A simple example appears as follows:
//! ```ignore
//! use lnk::ShellLink;
//! // ...
//! ShellLink::new_simple(std::path::Path::new(r"C:\Windows\System32\notepad.exe"));
//! ```
//!
//! > **IMPORTANT!**: Writing capability is currently in a very early stage and probably won't work!

use byteorder::{ByteOrder, LE};
#[allow(unused)]
use log::{debug, error, info, trace, warn};

use std::convert::TryFrom;
use std::fs::File;
#[cfg(feature = "experimental_save")]
use std::io::BufWriter;
use std::io::{prelude::*, BufReader};
#[cfg(feature = "experimental_save")]
use std::path::Path;

mod header;
pub use header::{
    FileAttributeFlags, HotkeyFlags, HotkeyKey, HotkeyModifiers, LinkFlags, ShellLinkHeader,
    ShowCommand,
};

/// The LinkTargetIDList structure specifies the target of the link. The presence of this optional
/// structure is specified by the HasLinkTargetIDList bit (LinkFlagssection 2.1.1) in the
/// ShellLinkHeader(section2.1).
pub mod linktarget;
pub use linktarget::LinkTargetIdList;

/// The LinkInfo structure specifies information necessary to resolve a
/// linktarget if it is not found in its original location. This includes
/// information about the volume that the target was stored on, the mapped
/// drive letter, and a Universal Naming Convention (UNC)form of the path
/// if one existed when the linkwas created. For more details about UNC
/// paths, see [MS-DFSNM] section 2.2.1.4
pub mod linkinfo;
pub use linkinfo::LinkInfo;

mod stringdata;

/// Structures from the ExtraData section of the Shell Link.
pub mod extradata;
pub use extradata::ExtraData;

mod filetime;
pub use filetime::FileTime;

mod strings;

/// The error type for shell link parsing errors.
#[derive(Debug)]
pub enum Error {
    /// An IO error occurred.
    IoError(std::io::Error),
    /// The parsed file isn't a shell link.
    NotAShellLinkError,
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IoError(e)
    }
}

/// A shell link
#[derive(Clone, Debug)]
pub struct ShellLink {
    shell_link_header: header::ShellLinkHeader,
    linktarget_id_list: Option<linktarget::LinkTargetIdList>,
    link_info: Option<linkinfo::LinkInfo>,
    name_string: Option<String>,
    relative_path: Option<String>,
    working_dir: Option<String>,
    command_line_arguments: Option<String>,
    icon_location: Option<String>,
    _extra_data: Vec<extradata::ExtraData>,
}

impl Default for ShellLink {
    /// Create a new ShellLink, left blank for manual configuration.
    /// For those who are not familar with the Shell Link specification, I
    /// suggest you look at the [`ShellLink::new_simple`] method.
    fn default() -> Self {
        Self {
            shell_link_header: header::ShellLinkHeader::default(),
            linktarget_id_list: None,
            link_info: None,
            name_string: None,
            relative_path: None,
            working_dir: None,
            command_line_arguments: None,
            icon_location: None,
            _extra_data: vec![],
        }
    }
}

impl ShellLink {
    #[cfg(feature = "experimental_save")]
    /// Create a new ShellLink pointing to a location, with otherwise default settings.
    pub fn new_simple<P: AsRef<Path>>(to: P) -> std::io::Result<Self> {
        use std::fs;

        let meta = fs::metadata(&to)?;
        let mut canonical = fs::canonicalize(&to)?.into_boxed_path();
        if cfg!(windows) {
            // Remove symbol for long path if present.
            let can_os = canonical.as_os_str().to_str().unwrap();
            if can_os.starts_with("\\\\?\\") {
                canonical = PathBuf::new().join(&can_os[4..]).into_boxed_path();
            }
        }

        let mut sl = Self::default();

        let mut flags = LinkFlags::IS_UNICODE;
        sl.header_mut().set_link_flags(flags);
        if meta.is_dir() {
            sl.header_mut()
                .set_file_attributes(FileAttributeFlags::FILE_ATTRIBUTE_DIRECTORY);
        } else {
            flags |= LinkFlags::HAS_WORKING_DIR
                | LinkFlags::HAS_RELATIVE_PATH
                | LinkFlags::HAS_LINK_INFO;
            sl.header_mut().set_link_flags(flags);
            sl.set_relative_path(Some(format!(
                ".\\{}",
                canonical.file_name().unwrap().to_str().unwrap()
            )));
            sl.set_working_dir(Some(
                canonical.parent().unwrap().to_str().unwrap().to_string(),
            ));
            sl.link_info = Some(_);
        }

        Ok(sl)
    }

    #[cfg(feature = "experimental_save")]
    /// Save a shell link.
    ///
    /// Note that this doesn't save any [`ExtraData`](struct.ExtraData.html) entries.
    pub fn save<P: AsRef<std::path::Path>>(&self, path: P) -> std::io::Result<()> {
        let mut w = BufWriter::new(File::create(path)?);

        debug!("Writing header...");
        let header_data: [u8; 0x4c] = self.shell_link_header.into();
        w.write_all(&header_data)?;

        let link_flags = *self.header().link_flags();

        if link_flags.contains(LinkFlags::HAS_LINK_TARGET_ID_LIST) {
            if let None = self.linktarget_id_list {
                error!("LinkTargetIDList not specified but expected!")
            }
            debug!("A LinkTargetIDList is marked as present. Writing.");
            let mut data: Vec<u8> = self.linktarget_id_list.clone().unwrap().into();
            w.write_all(&mut data)?;
        }

        if link_flags.contains(LinkFlags::HAS_LINK_INFO) {
            if let None = self.link_info {
                error!("LinkInfo not specified but expected!")
            }
            debug!("LinkInfo is marked as present. Writing.");
            let mut data: Vec<u8> = self.link_info.clone().unwrap().into();
            w.write_all(&mut data)?;
        }

        if link_flags.contains(LinkFlags::HAS_NAME) {
            if self.name_string == None {
                error!("Name not specified but expected!")
            }
            debug!("Name is marked as present. Writing.");
            w.write_all(&stringdata::to_data(
                self.name_string.as_ref().unwrap(),
                link_flags,
            ))?;
        }

        if link_flags.contains(LinkFlags::HAS_RELATIVE_PATH) {
            if self.relative_path == None {
                error!("Relative path not specified but expected!")
            }
            debug!("Relative path is marked as present. Writing.");
            w.write_all(&stringdata::to_data(
                self.relative_path.as_ref().unwrap(),
                link_flags,
            ))?;
        }

        if link_flags.contains(LinkFlags::HAS_WORKING_DIR) {
            if self.working_dir == None {
                error!("Working Directory not specified but expected!")
            }
            debug!("Working dir is marked as present. Writing.");
            w.write_all(&stringdata::to_data(
                self.working_dir.as_ref().unwrap(),
                link_flags,
            ))?;
        }

        if link_flags.contains(LinkFlags::HAS_ARGUMENTS) {
            if self.icon_location == None {
                error!("Arguments not specified but expected!")
            }
            debug!("Arguments are marked as present. Writing.");
            w.write_all(&stringdata::to_data(
                self.command_line_arguments.as_ref().unwrap(),
                link_flags,
            ))?;
        }

        if link_flags.contains(LinkFlags::HAS_ICON_LOCATION) {
            if self.icon_location == None {
                error!("Icon Location not specified but expected!")
            }
            debug!("Icon Location is marked as present. Writing.");
            w.write_all(&stringdata::to_data(
                self.icon_location.as_ref().unwrap(),
                link_flags,
            ))?;
        }

        Ok(())
    }

    /// Open and parse a shell link
    pub fn open<P: AsRef<std::path::Path>>(path: P) -> Result<Self, Error> {
        debug!("Opening {:?}", path.as_ref());
        let mut r = BufReader::new(File::open(path)?);
        let mut data = vec![];
        trace!("Reading file.");
        r.read_to_end(&mut data)?;

        trace!("Parsing shell header.");
        if data.len() < 0x4c {
            return Err(Error::NotAShellLinkError);
        }
        let shell_link_header = header::ShellLinkHeader::try_from(&data[0..0x4c])?;
        debug!("Shell header: {:#?}", shell_link_header);

        let mut cursor = 0x4c;

        let mut linktarget_id_list = None;
        let link_flags = *shell_link_header.link_flags();
        if link_flags.contains(LinkFlags::HAS_LINK_TARGET_ID_LIST) {
            debug!("A LinkTargetIDList is marked as present. Parsing now.");
            debug!("Cursor position: 0x{:x}", cursor);
            let list = linktarget::LinkTargetIdList::from(&data[cursor..]);
            debug!("{:?}", list);
            cursor += list.size as usize + 2; // add LinkTargetSize size
            linktarget_id_list = Some(list);
        }

        let mut link_info = None;
        if link_flags.contains(LinkFlags::HAS_LINK_INFO) {
            debug!("LinkInfo is marked as present. Parsing now.");
            debug!("Cursor position: 0x{:x}", cursor);
            let info = linkinfo::LinkInfo::from(&data[cursor..]);
            debug!("{:?}", info);
            cursor += info.size as usize;
            link_info = Some(info);
        }

        let mut name_string = None;
        let mut relative_path = None;
        let mut working_dir = None;
        let mut command_line_arguments = None;
        let mut icon_location = None;

        if link_flags.contains(LinkFlags::HAS_NAME) {
            debug!("Name is marked as present. Parsing now.");
            debug!("Cursor position: 0x{:x}", cursor);
            let (len, data) = stringdata::parse_string(&data[cursor..], link_flags);
            name_string = Some(data);
            cursor += len; // add len bytes
        }

        if link_flags.contains(LinkFlags::HAS_RELATIVE_PATH) {
            debug!("Relative path is marked as present. Parsing now.");
            debug!("Cursor position: 0x{:x}", cursor);
            let (len, data) = stringdata::parse_string(&data[cursor..], link_flags);
            relative_path = Some(data);
            cursor += len; // add len bytes
        }

        if link_flags.contains(LinkFlags::HAS_WORKING_DIR) {
            debug!("Working dir is marked as present. Parsing now.");
            debug!("Cursor position: 0x{:x}", cursor);
            let (len, data) = stringdata::parse_string(&data[cursor..], link_flags);
            working_dir = Some(data);
            cursor += len; // add len bytes
        }

        if link_flags.contains(LinkFlags::HAS_ARGUMENTS) {
            debug!("Arguments are marked as present. Parsing now.");
            debug!("Cursor position: 0x{:x}", cursor);
            let (len, data) = stringdata::parse_string(&data[cursor..], link_flags);
            command_line_arguments = Some(data);
            cursor += len; // add len bytes
        }

        if link_flags.contains(LinkFlags::HAS_ICON_LOCATION) {
            debug!("Icon Location is marked as present. Parsing now.");
            debug!("Cursor position: 0x{:x}", cursor);
            let (len, data) = stringdata::parse_string(&data[cursor..], link_flags);
            icon_location = Some(data);
            cursor += len; // add len bytes
        }

        let mut extra_data = Vec::new();

        loop {
            if data[cursor..].len() < 4 {
                warn!("The ExtraData length is invalid.");
                break; // Probably an error?
            }
            debug!("Parsing ExtraData");
            debug!("Cursor position: 0x{:x}", cursor);
            let query = LE::read_u32(&data[cursor..]);
            if query < 0x04 {
                break;
            }
            extra_data.push(extradata::ExtraData::from(&data[cursor..]));
            cursor += query as usize;
        }

        let _remaining_data = &data[cursor..];

        Ok(Self {
            shell_link_header,
            linktarget_id_list,
            link_info,
            name_string,
            relative_path,
            working_dir,
            command_line_arguments,
            icon_location,
            _extra_data: extra_data,
        })
    }

    /// Get the header of the shell link
    pub fn header(&self) -> &ShellLinkHeader {
        &self.shell_link_header
    }

    #[cfg(feature = "experimental_save")]
    /// Get a mutable instance of the shell link's header
    pub fn header_mut(&mut self) -> &mut ShellLinkHeader {
        &mut self.shell_link_header
    }

    /// Get the link target ID List
    pub fn link_target_id_list(&self) -> &Option<LinkTargetIdList> {
        &self.linktarget_id_list
    }

    /// Get the link info structure
    pub fn link_info(&self) -> &Option<LinkInfo> {
        &self.link_info
    }

    /// Get the shell link's name, if set
    pub fn name(&self) -> &Option<String> {
        &self.name_string
    }

    #[cfg(feature = "experimental_save")]
    /// Set the shell link's name
    pub fn set_name(&mut self, name: Option<String>) {
        self.header_mut()
            .update_link_flags(LinkFlags::HAS_NAME, name.is_some());
        self.name_string = name;
    }

    /// Get the shell link's relative path, if set
    pub fn relative_path(&self) -> &Option<String> {
        &self.relative_path
    }

    #[cfg(feature = "experimental_save")]
    /// Set the shell link's relative path
    pub fn set_relative_path(&mut self, relative_path: Option<String>) {
        self.header_mut()
            .update_link_flags(LinkFlags::HAS_RELATIVE_PATH, relative_path.is_some());
        self.relative_path = relative_path;
    }

    /// Get the shell link's working directory, if set
    pub fn working_dir(&self) -> &Option<String> {
        &self.working_dir
    }

    #[cfg(feature = "experimental_save")]
    /// Set the shell link's working directory
    pub fn set_working_dir(&mut self, working_dir: Option<String>) {
        self.header_mut()
            .update_link_flags(LinkFlags::HAS_WORKING_DIR, working_dir.is_some());
        self.working_dir = working_dir;
    }

    /// Get the shell link's arguments, if set
    pub fn arguments(&self) -> &Option<String> {
        &self.command_line_arguments
    }

    #[cfg(feature = "experimental_save")]
    /// Set the shell link's arguments
    pub fn set_arguments(&mut self, arguments: Option<String>) {
        self.header_mut()
            .update_link_flags(LinkFlags::HAS_ARGUMENTS, arguments.is_some());
        self.command_line_arguments = arguments;
    }

    /// Get the shell link's icon location, if set
    pub fn icon_location(&self) -> &Option<String> {
        &self.icon_location
    }

    #[cfg(feature = "experimental_save")]
    /// Set the shell link's icon location
    pub fn set_icon_location(&mut self, icon_location: Option<String>) {
        self.header_mut()
            .update_link_flags(LinkFlags::HAS_ICON_LOCATION, icon_location.is_some());
        self.icon_location = icon_location;
    }
}
