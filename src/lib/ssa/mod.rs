// Copyright (c) 2024-2026 Jonathan A. Thomason

pub mod builder;
pub mod data;

#[derive(Debug)]
pub enum Error {
	UnusedId,
	DetachedInstruction(data::Instruction),
	NestedFunction,
}
