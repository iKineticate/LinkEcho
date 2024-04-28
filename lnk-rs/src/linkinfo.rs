use bitflags::bitflags;
use byteorder::{ByteOrder, LE};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::FromPrimitive;

use crate::strings;

/// The LinkInfo structure specifies information necessary to resolve a
/// linktarget if it is not found in its original location. This includes
/// information about the volume that the target was stored on, the mapped
/// drive letter, and a Universal Naming Convention (UNC)form of the path
/// if one existed when the linkwas created. For more details about UNC
/// paths, see [MS-DFSNM] section 2.2.1.4
#[derive(Clone, Debug)]
pub struct LinkInfo {
    /// The parsed struct size
    pub size: u32,
    /// Flags that specify whether the VolumeID, LocalBasePath,
    /// LocalBasePathUnicode, and CommonNetworkRelativeLinkfields are present
    /// in this structure.
    _link_info_flags: LinkInfoFlags,
    /// An optional VolumeID structure (section 2.3.1) that specifies
    /// information about the volume that the link target was on when the link
    /// was created. This field is present if the VolumeIDAndLocalBasePath
    /// flag is set.
    volume_id: Option<VolumeID>,
    /// An optional, NULL–terminated string, defined by the system default code
    /// page, which is used to construct the full path to the link item or link
    /// target by appending the string in the CommonPathSuffix field. This
    /// field is present if the VolumeIDAndLocalBasePath flag is set.
    local_base_path: Option<String>,
    /// An optional CommonNetworkRelativeLink structure (section 2.3.2) that
    /// specifies information about the network location where the link target
    /// is stored.
    common_network_relative_link: Option<CommonNetworkRelativeLink>,
    /// A NULL–terminated string, defined by the system default code page,
    /// which is used to construct the full path to the link item or link
    /// target by being appended to the string in the LocalBasePath field.
    common_path_suffix: String,
    /// An optional, NULL–terminated, Unicode string that is used to construct
    /// the full path to the link item or link target by appending the string
    /// in the CommonPathSuffixUnicode field. This field can be present only
    /// if the VolumeIDAndLocalBasePath flag is set and the value of the
    /// LinkInfoHeaderSize field is greater than or equal to 0x00000024.
    local_base_path_unicode: Option<String>,
    /// An optional, NULL–terminated, Unicode string that is used to construct
    /// the full path to the link item or link target by being appended to the
    /// string in the LocalBasePathUnicode field. This field can be present
    /// only if the value of the LinkInfoHeaderSize field is greater than or
    /// equal to 0x00000024.
    common_path_suffix_unicode: Option<String>,
}

impl LinkInfo {
    /// An optional VolumeID structure (section 2.3.1) that specifies
    /// information about the volume that the link target was on when the link
    /// was created. This field is present if the VolumeIDAndLocalBasePath
    /// flag is set.
    pub fn volume_id(&self) -> &Option<VolumeID> {
        &self.volume_id
    }
    /// An optional, NULL–terminated string, defined by the system default code
    /// page, which is used to construct the full path to the link item or link
    /// target by appending the string in the CommonPathSuffix field. This
    /// field is present if the VolumeIDAndLocalBasePath flag is set.
    pub fn local_base_path(&self) -> &Option<String> {
        &self.local_base_path
    }
    /// An optional CommonNetworkRelativeLink structure (section 2.3.2) that
    /// specifies information about the network location where the link target
    /// is stored.
    pub fn common_network_relative_link(&self) -> &Option<CommonNetworkRelativeLink> {
        &self.common_network_relative_link
    }
    /// A NULL–terminated string, defined by the system default code page,
    /// which is used to construct the full path to the link item or link
    /// target by being appended to the string in the LocalBasePath field.
    pub fn common_path_suffix(&self) -> &String {
        &self.common_path_suffix
    }
    /// An optional, NULL–terminated, Unicode string that is used to construct
    /// the full path to the link item or link target by appending the string
    /// in the CommonPathSuffixUnicode field. This field can be present only
    /// if the VolumeIDAndLocalBasePath flag is set and the value of the
    /// LinkInfoHeaderSize field is greater than or equal to 0x00000024.
    pub fn local_base_path_unicode(&self) -> &Option<String> {
        &self.local_base_path_unicode
    }
    /// An optional, NULL–terminated, Unicode string that is used to construct
    /// the full path to the link item or link target by being appended to the
    /// string in the LocalBasePathUnicode field. This field can be present
    /// only if the value of the LinkInfoHeaderSize field is greater than or
    /// equal to 0x00000024.
    pub fn common_path_suffix_unicode(&self) -> &Option<String> {
        &self.common_path_suffix_unicode
    }
}

impl Default for LinkInfo {
    fn default() -> Self {
        Self {
            size: 0,
            _link_info_flags: LinkInfoFlags::empty(),
            volume_id: None,
            local_base_path: None,
            common_network_relative_link: None,
            common_path_suffix: String::new(),
            local_base_path_unicode: None,
            common_path_suffix_unicode: None,
        }
    }
}

impl From<&[u8]> for LinkInfo {
    fn from(data: &[u8]) -> Self {
        let mut link_info = Self::default();

        link_info.size = LE::read_u32(data);
        let header_size = LE::read_u32(&data[4..]);
        let extra_offsets_specified = header_size >= 0x24;
        let flags = LinkInfoFlags::from_bits_truncate(LE::read_u32(&data[8..]));
        let volume_id_offset = LE::read_u32(&data[12..]) as usize;
        let local_base_path_offset = LE::read_u32(&data[16..]) as usize;
        let common_network_relative_link_offset = LE::read_u32(&data[20..]) as usize;
        let common_path_suffix_offset = LE::read_u32(&data[24..]) as usize;
        let mut local_base_path_offset_unicode = 0;
        if extra_offsets_specified {
            local_base_path_offset_unicode = LE::read_u32(&data[28..]) as usize;
            let common_path_suffix_offset_unicode = LE::read_u32(&data[32..]) as usize;

            if common_path_suffix_offset_unicode != 0 {
                link_info.common_path_suffix_unicode = Some(strings::trim_nul_terminated_string(
                    String::from_utf8_lossy(&data[common_path_suffix_offset_unicode..]).to_string(),
                ));
            }
        }
        if flags & LinkInfoFlags::VOLUME_ID_AND_LOCAL_BASE_PATH
            == LinkInfoFlags::VOLUME_ID_AND_LOCAL_BASE_PATH
        {
            assert_ne!(volume_id_offset, 0);
            assert_ne!(local_base_path_offset, 0);
            link_info.volume_id = Some(VolumeID::from(&data[volume_id_offset..]));
            link_info.local_base_path = Some(strings::trim_nul_terminated_string(
                String::from_utf8_lossy(&data[local_base_path_offset..]).to_string(),
            ));

            if local_base_path_offset_unicode != 0 {
                link_info.local_base_path_unicode = Some(strings::trim_nul_terminated_string(
                    String::from_utf8_lossy(&data[local_base_path_offset_unicode..]).to_string(),
                ));
            }
        }
        if flags & LinkInfoFlags::COMMON_NETWORK_RELATIVE_LINK_AND_PATH_SUFFIX
            == LinkInfoFlags::COMMON_NETWORK_RELATIVE_LINK_AND_PATH_SUFFIX
        {
            assert_ne!(common_network_relative_link_offset, 0);
            link_info.common_network_relative_link = Some(CommonNetworkRelativeLink::from(
                &data[common_network_relative_link_offset..],
            ));
        }
        link_info.common_path_suffix = strings::trim_nul_terminated_string(
            String::from_utf8_lossy(&data[common_path_suffix_offset..]).to_string(),
        );

        link_info
    }
}

impl Into<Vec<u8>> for LinkInfo {
    fn into(self) -> Vec<u8> {
        unimplemented!()
    }
}

bitflags! {
    /// Flags that specify whether the VolumeID, LocalBasePath, LocalBasePathUnicode,
    /// and CommonNetworkRelativeLink fields are present in this structure.
    pub struct LinkInfoFlags: u32 {
        /// If set, the VolumeIDand LocalBasePath fields are present, and their
        /// locations are specified by the values of the VolumeIDOffset and
        /// LocalBasePathOffset fields, respectively. If the value of the
        /// LinkInfoHeaderSize field is greater than or equal to 0x00000024, the
        /// LocalBasePathUnicode field is present, and its location is specified
        /// by the value of the LocalBasePathOffsetUnicode field. If not set,
        /// the VolumeID, LocalBasePath, and LocalBasePathUnicode fields are
        /// not present, and the values of the VolumeIDOffset and
        /// LocalBasePathOffset fields are zero. If the value of the
        /// LinkInfoHeaderSize field is greater than or equal to 0x00000024,
        /// the value of the LocalBasePathOffsetUnicode field is zero.
        const VOLUME_ID_AND_LOCAL_BASE_PATH = 0b0000_0000_0000_0000_0000_0000_0000_0001;

        /// If set, the CommonNetworkRelativeLink field is present, and its
        /// location is specified by the value of the
        /// CommonNetworkRelativeLinkOffset field.If not set, the
        /// CommonNetworkRelativeLink field is not present, and the value of
        /// the CommonNetworkRelativeLinkOffset field is zero
        const COMMON_NETWORK_RELATIVE_LINK_AND_PATH_SUFFIX = 0b0000_0000_0000_0000_0000_0000_0000_0010;
    }
}

/// The VolumeID structure specifies information about the volume that a link
/// target was on when the link was created. This information is useful for
/// resolving the link if the file is not found in its original location.
#[derive(Clone, Debug)]
pub struct VolumeID {
    /// A 32-bit, unsigned integer that specifies the type of drive the link
    /// target is stored on.
    drive_type: DriveType,
    /// A 32-bit, unsigned integer that specifies the drive serial number of
    /// the volume the link target is stored on.
    drive_serial_number: u32,
    /// The label of the volume that the link target is stored on.
    volume_label: String,
}

impl VolumeID {
    /// A 32-bit, unsigned integer that specifies the type of drive the link
    /// target is stored on.
    pub fn drive_type(&self) -> &DriveType {
        &self.drive_type
    }
    /// A 32-bit, unsigned integer that specifies the drive serial number of
    /// the volume the link target is stored on.
    pub fn drive_serial_number(&self) -> &u32 {
        &self.drive_serial_number
    }
    /// The label of the volume that the link target is stored on.
    pub fn volume_label(&self) -> &String {
        &self.volume_label
    }
}

impl Default for VolumeID {
    fn default() -> Self {
        Self {
            drive_type: DriveType::DriveUnknown,
            drive_serial_number: 0,
            volume_label: String::new(),
        }
    }
}

impl From<&[u8]> for VolumeID {
    fn from(data: &[u8]) -> Self {
        let mut volume_id = VolumeID::default();

        let _size = LE::read_u32(data);
        volume_id.drive_type = DriveType::from_u32(LE::read_u32(&data[4..])).unwrap();
        volume_id.drive_serial_number = LE::read_u32(&data[8..]);
        let mut volume_label_offset = LE::read_u32(&data[12..]) as usize;
        if volume_label_offset == 0x14 {
            volume_label_offset /* _unicode */ = LE::read_u32(&data[16..]) as usize;
        }
        volume_id.volume_label = strings::trim_nul_terminated_string(
            String::from_utf8_lossy(&data[volume_label_offset..]).to_string(),
        );

        volume_id
    }
}

impl Into<Vec<u8>> for VolumeID {
    fn into(self) -> Vec<u8> {
        unimplemented!()
    }
}

/// A 32-bit, unsigned integer that specifies the type of drive the link target is stored on.
#[derive(Clone, Debug, FromPrimitive, ToPrimitive)]
pub enum DriveType {
    /// The drive type cannot be determined.
    DriveUnknown = 0x00,
    /// The root path is invalid; for example, there is no volume mounted at the path.
    DriveNoRootDir = 0x01,
    /// The drive has removable media, such as a floppy drive, thumb drive, or flash card reader.
    DriveRemovable = 0x02,
    /// The drive has fixed media, such as a hard drive or flash drive.
    DriveFixed = 0x03,
    /// The drive is a remote (network) drive.
    DriveRemote = 0x04,
    /// The drive is a CD-ROM drive.
    DriveCDRom = 0x05,
    /// The drive is a RAM disk.
    DriveRamdisk = 0x06,
}

/// The CommonNetworkRelativeLink structure specifies information about the network location where a
/// link target is stored, including the mapped drive letter and the UNC path prefix. For details on
/// UNC paths, see [MS-DFSNM] section 2.2.1.4.
#[derive(Clone, Debug)]
pub struct CommonNetworkRelativeLink {
    /// Flags that specify the contents of the DeviceNameOffset and
    /// NetProviderType fields.
    flags: CommonNetworkRelativeLinkFlags,
    /// A 32-bit, unsigned integer that specifies the type of network
    /// provider.
    network_provider_type: Option<NetworkProviderType>,
    /// A NULL–terminated string, as defined by the system default code
    /// page, which specifies a server share path; for example,
    /// "\\server\share".
    net_name: String,
    /// A NULL–terminated string, as defined by the system default code
    /// page, which specifies a device; for example, the drive letter
    /// "D:".
    device_name: String,
    /// An optional, NULL–terminated, Unicode string that is the
    /// Unicode version of the NetName string. This field MUST be
    /// present if the value of the NetNameOffset field is greater
    /// than 0x00000014; otherwise, this field MUST NOT be present.
    net_name_unicode: Option<String>,
    /// An optional, NULL–terminated, Unicode string that is the
    /// Unicode version of the DeviceName string. This field MUST be
    /// present if the value of the NetNameOffset field is greater than
    /// 0x00000014; otherwise, this field MUST NOT be present.
    device_name_unicode: Option<String>,
}

impl Default for CommonNetworkRelativeLink {
    fn default() -> Self {
        Self {
            flags: CommonNetworkRelativeLinkFlags::empty(),
            network_provider_type: None,
            net_name: String::new(),
            device_name: String::new(),
            net_name_unicode: None,
            device_name_unicode: None,
        }
    }
}

impl From<&[u8]> for CommonNetworkRelativeLink {
    fn from(data: &[u8]) -> Self {
        let mut link = CommonNetworkRelativeLink::default();

        let size = LE::read_u32(data);
        assert!(size >= 0x14);
        link.flags = CommonNetworkRelativeLinkFlags::from_bits_truncate(LE::read_u32(&data[4..]));
        let net_name_offset = LE::read_u32(&data[8..]) as usize;
        let device_name_offset = LE::read_u32(&data[12..]) as usize;
        if link.flags & CommonNetworkRelativeLinkFlags::VALID_NET_TYPE
            == CommonNetworkRelativeLinkFlags::VALID_NET_TYPE
        {
            link.network_provider_type = NetworkProviderType::from_u32(LE::read_u32(&data[16..]));
        }
        link.net_name = strings::trim_nul_terminated_string(
            String::from_utf8_lossy(&data[net_name_offset..]).to_string(),
        );
        link.device_name = strings::trim_nul_terminated_string(
            String::from_utf8_lossy(&data[device_name_offset..]).to_string(),
        );
        if net_name_offset >= 0x14 {
            let net_name_offset_unicode = LE::read_u32(&data[20..]) as usize;
            let device_name_offset_unicode = LE::read_u32(&data[24..]) as usize;
            link.net_name_unicode = Some(strings::trim_nul_terminated_string(
                String::from_utf8_lossy(&data[net_name_offset_unicode..]).to_string(),
            ));
            link.device_name_unicode = Some(strings::trim_nul_terminated_string(
                String::from_utf8_lossy(&data[device_name_offset_unicode..]).to_string(),
            ));
        }

        link
    }
}

impl Into<Vec<u8>> for CommonNetworkRelativeLink {
    fn into(self) -> Vec<u8> {
        unimplemented!()
    }
}

bitflags! {
    /// Flags that specify the contents of the DeviceNameOffset and NetProviderType fields.
    pub struct CommonNetworkRelativeLinkFlags: u32 {
        /// If set, the DeviceNameOffset field contains an offset to the device
        /// name. If not set, the DeviceNameOffset field does not contain an
        /// offset to the device name, and its value MUST be zero.
        const VALID_DEVICE = 0b0000_0000_0000_0000_0000_0000_0000_0001;
        /// If set, the NetProviderType field contains the network provider
        /// type. If not set, the NetProviderType field does not contain the
        /// network provider type, and its value MUST be zero.
        const VALID_NET_TYPE = 0b0000_0000_0000_0000_0000_0000_0000_0010;
    }
}

/// A 32-bit, unsigned integer that specifies the type of network provider.
#[allow(missing_docs)]
#[derive(Clone, Debug, FromPrimitive, ToPrimitive)]
pub enum NetworkProviderType {
    Avid = 0x1a0000,
    Docuspace = 0x1b0000,
    Mangosoft = 0x1c0000,
    Sernet = 0x1d0000,
    Riverfront1 = 0x1e0000,
    Riverfront2 = 0x1f0000,
    Decorb = 0x200000,
    Protstor = 0x210000,
    FjRedir = 0x220000,
    Distinct = 0x230000,
    Twins = 0x240000,
    Rdr2Sample = 0x250000,
    CSC = 0x260000,
    _3In1 = 0x270000,
    ExtendNet = 0x290000,
    Stac = 0x2a0000,
    Foxbat = 0x2b0000,
    Yahoo = 0x2c0000,
    Exifs = 0x2d0000,
    Dav = 0x2e0000,
    Knoware = 0x2f0000,
    ObjectDire = 0x300000,
    Masfax = 0x310000,
    HobNfs = 0x320000,
    Shiva = 0x330000,
    Ibmal = 0x340000,
    Lock = 0x350000,
    Termsrv = 0x360000,
    Srt = 0x370000,
    Quincy = 0x380000,
    Openafs = 0x390000,
    Avid1 = 0x3a0000,
    Dfs = 0x3b0000,
    Kwnp = 0x3c0000,
    Zenworks = 0x3d0000,
    Driveonweb = 0x3e0000,
    Vmware = 0x3f0000,
    Rsfx = 0x400000,
    Mfiles = 0x410000,
    MsNfs = 0x420000,
    Google = 0x430000,
}
