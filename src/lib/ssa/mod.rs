// Copyright (c) 2024-2025 Jonathan Thomason

pub mod build;
pub mod data;

pub enum Error {
	UnusedId,
	DetachedInstruction(Option<data::Instruction>),
	NestedFunction,
}
