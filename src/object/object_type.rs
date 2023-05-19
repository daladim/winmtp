use windows::core::GUID;
use windows::Win32::Devices::PortableDevices::{
    WPD_CONTENT_TYPE_APPOINTMENT, WPD_CONTENT_TYPE_AUDIO, WPD_CONTENT_TYPE_AUDIO_ALBUM, WPD_CONTENT_TYPE_CALENDAR,
    WPD_CONTENT_TYPE_CERTIFICATE, WPD_CONTENT_TYPE_CONTACT, WPD_CONTENT_TYPE_CONTACT_GROUP, WPD_CONTENT_TYPE_DOCUMENT,
    WPD_CONTENT_TYPE_EMAIL, WPD_CONTENT_TYPE_FOLDER, WPD_CONTENT_TYPE_FUNCTIONAL_OBJECT, WPD_CONTENT_TYPE_GENERIC_FILE,
    WPD_CONTENT_TYPE_GENERIC_MESSAGE, WPD_CONTENT_TYPE_IMAGE, WPD_CONTENT_TYPE_IMAGE_ALBUM, WPD_CONTENT_TYPE_MEDIA_CAST,
    WPD_CONTENT_TYPE_MEMO, WPD_CONTENT_TYPE_MIXED_CONTENT_ALBUM, WPD_CONTENT_TYPE_NETWORK_ASSOCIATION,
    WPD_CONTENT_TYPE_PLAYLIST, WPD_CONTENT_TYPE_PROGRAM, WPD_CONTENT_TYPE_SECTION, WPD_CONTENT_TYPE_TASK,
    WPD_CONTENT_TYPE_TELEVISION, WPD_CONTENT_TYPE_UNSPECIFIED, WPD_CONTENT_TYPE_VIDEO, WPD_CONTENT_TYPE_VIDEO_ALBUM,
    WPD_CONTENT_TYPE_WIRELESS_PROFILE, WPD_CONTENT_TYPE_ALL,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObjectType {
    All,
    Appointment,
    Audio,
    AudioAlbum,
    Calendar,
    Certificate,
    Contact,
    ContactGroup,
    Document,
    Email,
    Folder,
    FunctionalObject,
    GenericFile,
    GenericMessage,
    Image,
    ImageAlbum,
    MediaCast,
    Memo,
    MixedContentAlbum,
    NetworkAssociation,
    Playlist,
    Program,
    Section,
    Task,
    Television,
    Unspecified,
    Video,
    VideoAlbum,
    WirelessProfile,
    Unknown,
}

impl ObjectType {
    pub fn from_guid(guid: GUID) -> Self {
        match guid {
            WPD_CONTENT_TYPE_APPOINTMENT            => Self::Appointment,
            WPD_CONTENT_TYPE_AUDIO                  => Self::Audio,
            WPD_CONTENT_TYPE_AUDIO_ALBUM            => Self::AudioAlbum,
            WPD_CONTENT_TYPE_CALENDAR               => Self::Calendar,
            WPD_CONTENT_TYPE_CERTIFICATE            => Self::Certificate,
            WPD_CONTENT_TYPE_CONTACT                => Self::Contact,
            WPD_CONTENT_TYPE_CONTACT_GROUP          => Self::ContactGroup,
            WPD_CONTENT_TYPE_DOCUMENT               => Self::Document,
            WPD_CONTENT_TYPE_EMAIL                  => Self::Email,
            WPD_CONTENT_TYPE_FOLDER                 => Self::Folder,
            WPD_CONTENT_TYPE_FUNCTIONAL_OBJECT      => Self::FunctionalObject,
            WPD_CONTENT_TYPE_GENERIC_FILE           => Self::GenericFile,
            WPD_CONTENT_TYPE_GENERIC_MESSAGE        => Self::GenericMessage,
            WPD_CONTENT_TYPE_IMAGE                  => Self::Image,
            WPD_CONTENT_TYPE_IMAGE_ALBUM            => Self::ImageAlbum,
            WPD_CONTENT_TYPE_MEDIA_CAST             => Self::MediaCast,
            WPD_CONTENT_TYPE_MEMO                   => Self::Memo,
            WPD_CONTENT_TYPE_MIXED_CONTENT_ALBUM    => Self::MixedContentAlbum,
            WPD_CONTENT_TYPE_NETWORK_ASSOCIATION    => Self::NetworkAssociation,
            WPD_CONTENT_TYPE_PLAYLIST               => Self::Playlist,
            WPD_CONTENT_TYPE_PROGRAM                => Self::Program,
            WPD_CONTENT_TYPE_SECTION                => Self::Section,
            WPD_CONTENT_TYPE_TASK                   => Self::Task,
            WPD_CONTENT_TYPE_TELEVISION             => Self::Television,
            WPD_CONTENT_TYPE_UNSPECIFIED            => Self::Unspecified,
            WPD_CONTENT_TYPE_VIDEO                  => Self::Video,
            WPD_CONTENT_TYPE_VIDEO_ALBUM            => Self::VideoAlbum,
            WPD_CONTENT_TYPE_WIRELESS_PROFILE       => Self::WirelessProfile,
            _                                       => Self::Unknown,
        }
    }

    pub fn as_guid(&self) -> GUID {
        match self {
            Self::Appointment        => WPD_CONTENT_TYPE_APPOINTMENT,
            Self::Audio              => WPD_CONTENT_TYPE_AUDIO,
            Self::AudioAlbum         => WPD_CONTENT_TYPE_AUDIO_ALBUM,
            Self::Calendar           => WPD_CONTENT_TYPE_CALENDAR,
            Self::Certificate        => WPD_CONTENT_TYPE_CERTIFICATE,
            Self::Contact            => WPD_CONTENT_TYPE_CONTACT,
            Self::ContactGroup       => WPD_CONTENT_TYPE_CONTACT_GROUP,
            Self::Document           => WPD_CONTENT_TYPE_DOCUMENT,
            Self::Email              => WPD_CONTENT_TYPE_EMAIL,
            Self::Folder             => WPD_CONTENT_TYPE_FOLDER,
            Self::FunctionalObject   => WPD_CONTENT_TYPE_FUNCTIONAL_OBJECT,
            Self::GenericFile        => WPD_CONTENT_TYPE_GENERIC_FILE,
            Self::GenericMessage     => WPD_CONTENT_TYPE_GENERIC_MESSAGE,
            Self::Image              => WPD_CONTENT_TYPE_IMAGE,
            Self::ImageAlbum         => WPD_CONTENT_TYPE_IMAGE_ALBUM,
            Self::MediaCast          => WPD_CONTENT_TYPE_MEDIA_CAST,
            Self::Memo               => WPD_CONTENT_TYPE_MEMO,
            Self::MixedContentAlbum  => WPD_CONTENT_TYPE_MIXED_CONTENT_ALBUM,
            Self::NetworkAssociation => WPD_CONTENT_TYPE_NETWORK_ASSOCIATION,
            Self::Playlist           => WPD_CONTENT_TYPE_PLAYLIST,
            Self::Program            => WPD_CONTENT_TYPE_PROGRAM,
            Self::Section            => WPD_CONTENT_TYPE_SECTION,
            Self::Task               => WPD_CONTENT_TYPE_TASK,
            Self::Television         => WPD_CONTENT_TYPE_TELEVISION,
            Self::Unspecified        => WPD_CONTENT_TYPE_UNSPECIFIED,
            Self::Video              => WPD_CONTENT_TYPE_VIDEO,
            Self::VideoAlbum         => WPD_CONTENT_TYPE_VIDEO_ALBUM,
            Self::WirelessProfile    => WPD_CONTENT_TYPE_WIRELESS_PROFILE,
            Self::All                => WPD_CONTENT_TYPE_ALL,    // Not sure about this one...
            Self::Unknown            => WPD_CONTENT_TYPE_ALL,    // Not sure about this one...
        }
    }

    /// Returns whether this object matches my opiniated way of seeing what a file should be like
    pub fn is_file_like(&self) -> bool {
        return false == matches!(self,
            Self::AudioAlbum |
            Self::ContactGroup |
            Self::Folder |
            Self::FunctionalObject |
            Self::ImageAlbum |
            Self::MixedContentAlbum |
            Self::VideoAlbum
        )
    }
}
