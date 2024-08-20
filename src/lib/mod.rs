// Copyright (c) 2024-2026 Jonathan A. Thomason

use std::{
	fmt,
	str::SplitTerminator,
};

use bitflags::bitflags;

pub mod asm;
pub mod lnk;
pub mod ssa;

bitflags! {
	#[derive(Debug, Clone, Copy)]
	pub struct StacklFlags: u32 {
		const FEATURE_GEN_IO   = 1;
		const FEATURE_PIO_TERM = 1 << 1;
		const FEATURE_DMA_TERM = 1 << 2;
		const FEATURE_DISK     = 1 << 3;
		const FEATURE_INP      = 1 << 4;
		const _ = !0;
	}
}

#[derive(Debug)]
pub struct Version(pub u32);
impl Version {
	pub fn new(variant: u32, major: u32, minor: u32, patch: u32) -> Self {
		Self((variant << 29) | (major << 22) | (minor << 12) | patch)
	}
	pub fn variant(&self) -> u32 {
		self.0 >> 29
	}
	pub fn major(&self) -> u32 {
		(self.0 >> 22) & 0x7fu32
	}
	pub fn minor(&self) -> u32 {
		(self.0 >> 12) & 0x3ff
	}
	pub fn patch(&self) -> u32 {
		self.0 & 0xfff
	}
}

impl fmt::Display for Version {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let variant = self.variant();
		let major = self.major();
		let minor = self.minor();
		let patch = self.patch();
		if variant == 0 {
			write!(f, "{major}.{minor}.{patch}")
		} else {
			write!(f, "({variant}) {major}.{minor}.{patch}")
		}
	}
}

#[derive(Debug)]
pub struct StacklFormatV1 {
	pub header: String,
	pub text: Vec<u8>,
}

impl StacklFormatV1 {
	pub fn version(&self) -> Option<String> {
		let mut token_iter = self.iter_header();
		for token in &mut token_iter {
			if token == "stackl" {
				return token_iter.next().map(|x| x.to_owned());
			}
		}
		None
	}
	pub fn flags(&self) -> Result<StacklFlags, ErrorKind> {
		let mut flags = StacklFlags::empty();
		let mut token_iter = self.iter_header();
		while let Some(token) = token_iter.next() {
			if token == "feature" {
				let Some(feature) = token_iter.next() else {
					return Err(ErrorKind::InvalidFeature);
				};
				match feature {
					"gen_io" => flags.set(StacklFlags::FEATURE_GEN_IO, true),
					"pio_term" => flags.set(StacklFlags::FEATURE_PIO_TERM, true),
					"dma_term" => flags.set(StacklFlags::FEATURE_DMA_TERM, true),
					"disk" => flags.set(StacklFlags::FEATURE_DISK, true),
					"inp" => flags.set(StacklFlags::FEATURE_INP, true),
					_ => return Err(ErrorKind::InvalidFeature),
				}
			}
		}
		Ok(flags)
	}
	pub fn stack_size(&self) -> Result<i32, ErrorKind> {
		let mut token_iter = self.iter_header();
		while let Some(token) = token_iter.next() {
			if token == "stack_size" {
				return token_iter
					.next()
					.and_then(|value| value.parse().ok())
					.ok_or(ErrorKind::InvalidStackSize);
			}
		}
		// default value
		Ok(1000)
	}
	fn iter_header(&self) -> SplitTerminator<'_, &[char]> {
		self.header.split_terminator(&['\n', ' '][..])
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
	InvalidVersion { expected: Version, found: Version },
	InvalidMagic,
	InvalidFeature,
	InvalidStackSize,
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
		if current_version.major() != 1 && current_version.variant() != 0 {
			return Err(ErrorKind::InvalidVersion {
				found: current_version,
				expected: Version::new(0, 1, 0, 0),
			});
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
			version: Version::new(1, 1, 0, 0),
			flags: value.flags()?,
			stack_size: value.stack_size()?,
			text: value.text,
		})
	}
}
