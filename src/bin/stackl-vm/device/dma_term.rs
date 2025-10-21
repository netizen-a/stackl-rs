use crate::machine::MachineState;
use std::sync::{
	Once,
	RwLock,
};

pub fn run_device(_machine_lock: &RwLock<MachineState>, _state: &Once) {
	todo!()
}
