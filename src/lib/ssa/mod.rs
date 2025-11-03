pub mod build;
pub mod data;

pub enum Error {
	UnusedId,
	DetachedInstruction(Option<data::Instruction>),
}
