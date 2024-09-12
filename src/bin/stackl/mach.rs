use std::sync::Mutex;

use crate::op;
pub struct MachineState {
    bp: i32,
    lp: i32,
    ip: i32,
    sp: i32,
    fp: i32,
    flag: i32,
    ivec: i32,
    vmem: i32,
    ram: Mutex<Vec<u8>>,
}

impl MachineState {
    pub fn new(mem_size: i32) -> MachineState {
        MachineState {
            bp: 0,
            lp: mem_size,
            ip: 0,
            sp: 0,
            fp: 0,
            flag: 0,
            ivec: 0,
            vmem: 0,
            ram: Mutex::new(vec![0x79; mem_size.try_into().unwrap()]),
        }
    }
    // returns true if success, else false
    pub fn store(&self, val: &[u8], offset: usize) -> bool {
        let mut ram = self.ram.lock().unwrap();
        if ram.len() > val.len() + offset {
            let count = val.len();
            unsafe {
                ram.as_mut_ptr()
                    .add(offset)
                    .copy_from_nonoverlapping(val.as_ptr(), count);
            }
            true
        } else {
            false
        }
    }
    pub fn execute(mut self) {
        loop {
            let ip: usize = self.ip.try_into().unwrap();
            let ram = self.ram.lock().unwrap();
            let op: u32 = u32::from_le_bytes(ram[ip..=(ip + 3)].try_into().unwrap());
            match op {
                op::NOP => {}
                op::PUSH => {
                    println!("push");
                    self.ip += 4;
                }
                op::POP => {println!("pop")}
                op::HALT => {
                    return;
                }
                k => unimplemented!("opcode {k}"),
            }
            self.ip += 4;
        }
    }
}
