pub mod build;
pub mod data;

pub enum Error {
    DetachedInstruction(Option<data::Instruction>),
}
