// Copyright (c) 2024-2026 Jonathan A. Thomason

use crate::machine::MachineState;
use std::sync::{
	Once,
	RwLock,
};

pub fn run_device(_machine_lock: &RwLock<MachineState>, _state: &Once) {
	todo!()
}
