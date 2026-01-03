// Copyright (c) 2024-2026 Jonathan Thomason

pub mod build;
pub mod data;

#[derive(Debug)]
pub enum Error {
	UnusedId,
	DetachedInstruction(Option<data::Instruction>),
	NestedFunction,
}
