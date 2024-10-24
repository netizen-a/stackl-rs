use bitflags::bitflags;

pub mod ast;
pub mod op;

bitflags! {
    #[derive(Debug)]
    pub struct StacklFlags: u32 {
        const LEGACY_MODE      = 1;
        const FEATURE_GEN_IO   = 1 << 1;
        const FEATURE_PIO_TERM = 1 << 2;
        const FEATURE_DMA_TERM = 1 << 3;
        const FEATURE_DISK     = 1 << 4;
        const FEATURE_INP      = 1 << 5;
        const _ = !0;
    }
}

#[derive(Debug)]
pub struct Version(pub u32);
impl Version {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self((major << 22) | (minor << 12) | patch)
    }
    pub fn major(&self) -> u32 {
        self.0 >> 22
    }
    pub fn minor(&self) -> u32 {
        (self.0 >> 12) & 0x3ff
    }
    pub fn patch(&self) -> u32 {
        self.0 & 0xfff
    }
}

#[derive(Debug)]
pub struct StacklFormatV1 {
    pub header: String,
    pub text: Vec<u8>,
}

impl StacklFormatV1 {
    pub fn version(&self) -> Option<Version> {
        // TODO: parse version
        Some(Version::new(1, 0, 0))
    }
    pub fn flags(&self) -> StacklFlags {
        let mut flags = StacklFlags::empty();
        flags.set(StacklFlags::LEGACY_MODE, true);
        flags
    }
    pub fn stack_size(&self) -> i32 {
        0
    }
}

impl TryFrom<&[u8]> for StacklFormatV1 {
    type Error = ErrorKind;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let Some(magic) = value.first_chunk::<6>() else {
            return Err(ErrorKind::InvalidMagic);
        };
        if magic != b"stackl" {
            return Err(ErrorKind::InvalidMagic);
        }

        let delim = b"begindata\n";
        let Some(delim_pos) = value
            .windows(delim.len())
            .position(|subslice| subslice == delim)
        else {
            return Err(ErrorKind::UnexpectedEof);
        };
        let (head, foot) = value.split_at(delim_pos + delim.len());
        let header = String::from_utf8_lossy(head).into_owned();
        Ok(StacklFormatV1 {
            header,
            text: foot.to_vec(),
        })
    }
}

#[derive(Debug)]
pub struct StacklFormatV2 {
    pub magic: [u8; 4],
    pub version: Version,
    pub flags: StacklFlags,
    pub stack_size: i32,
    pub text: Vec<u8>,
}

impl StacklFormatV2 {
    pub fn to_vec(self) -> Vec<u8> {
        let mut ret = Vec::from(self.magic);
        ret.extend(self.version.0.to_le_bytes());
        ret.extend(self.flags.bits().to_le_bytes());
        ret.extend(self.stack_size.to_le_bytes());
        ret.extend(self.text);
        ret
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    UnexpectedEof,
    InvalidVersion,
    InvalidMagic,
}

impl TryFrom<&[u8]> for StacklFormatV2 {
    type Error = ErrorKind;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < 20 {
            return Err(ErrorKind::UnexpectedEof);
        }
        let magic: [u8; 4] = value[..4].try_into().unwrap();
        let version: u32 = u32::from_le_bytes(value[4..8].try_into().unwrap());
        let flags = u32::from_le_bytes(value[8..12].try_into().unwrap());
        let stack_size = i32::from_le_bytes(value[12..16].try_into().unwrap());

        if magic != [b's', b'l', 0, 0] {
            return Err(ErrorKind::InvalidMagic);
        }
        let current_version = Version(version);
        let expected_version = Version::new(2, 0, 0);
        if current_version.major() != expected_version.major() {
            return Err(ErrorKind::InvalidVersion);
        }

        Ok(StacklFormatV2 {
            magic,
            version: Version(version),
            flags: StacklFlags::from_bits_retain(flags),
            stack_size,
            text: Vec::from(&value[16..]),
        })
    }
}

impl TryFrom<StacklFormatV1> for StacklFormatV2 {
    type Error = ErrorKind;
    fn try_from(value: StacklFormatV1) -> Result<Self, Self::Error> {
        Ok(StacklFormatV2 {
            magic: [b's', b'l', 0, 0],
            version: value.version().unwrap(),
            flags: value.flags(),
            stack_size: value.stack_size(),
            text: value.text,
        })
    }
}
