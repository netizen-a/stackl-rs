// Copyright (c) 2024-2026 Jonathan A. Thomason

use std::{
	fmt,
	path::PathBuf,
};

use clap::Parser;

#[derive(Debug, Clone, clap::ValueEnum, Default)]
pub enum EnableColor {
	#[default]
	Auto,
	Always,
	Never,
}

impl fmt::Display for EnableColor {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let s = match self {
			Self::Auto => "auto",
			Self::Always => "never",
			Self::Never => "never",
		};
		write!(f, "{s}")
	}
}

#[derive(Debug, Clone, Copy, clap::ValueEnum, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum WarnLevel {
	#[default]
	None,
	Minimal,
	All,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OptLevel {
	#[default]
	None,
	Standard,
}

impl OptLevel {
	#[allow(dead_code)]
	#[inline]
	pub fn is_none(&self) -> bool {
		matches!(self, Self::None)
	}
}

impl fmt::Display for OptLevel {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let s = match self {
			Self::None => "0",
			Self::Standard => "1",
		};
		write!(f, "{s}")
	}
}

impl clap::ValueEnum for OptLevel {
	fn value_variants<'a>() -> &'a [Self] {
		static OPTIONS: [OptLevel; 2] = [OptLevel::None, OptLevel::Standard];
		&OPTIONS
	}
	fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
		match self {
			Self::None => Some(clap::builder::PossibleValue::new("0")),
			Self::Standard => Some(clap::builder::PossibleValue::new("1")),
		}
	}
}

#[derive(Parser, Debug)]
#[command(version, about = "Stackl C99 compiler", long_about = None)]
pub struct Args {
	#[arg(name = "FILE", required = true)]
	pub in_file: PathBuf,
	#[arg(long = "output", short = 'o')]
	pub out_file: Option<PathBuf>,
	#[arg(
		short = 'E',
		group = "early-exit",
		group = "stdout",
		help = "Preprocess only; do not compile, assemble or link"
	)]
	pub stdout_preproc: bool,
	#[arg(long = "trace", group = "stdout")]
	pub is_traced: bool,
	#[arg(value_enum, long, default_value_t = Default::default())]
	pub enable_color: EnableColor,
	#[arg(short = 'W', value_enum, default_value_t = Default::default())]
	pub warn_lvl: WarnLevel,
	#[arg(
		short = 'S',
		group = "early-exit",
		help = "Compile only; do not assemble or link"
	)]
	pub check: bool,
	#[arg(
		long = "time",
		group = "stdout",
		help = "Time the execution of each subprocess"
	)]
	pub is_timed: bool,
	#[arg(short = 'O', help = "Optimize", default_value_t = Default::default())]
	pub opt_lvl: OptLevel,
	#[arg(long, group = "stdout", help = "prints ast")]
	pub ast: bool,
	#[arg(short = 'g', help = "Generate debug information")]
	pub gen_debug: bool,
}
