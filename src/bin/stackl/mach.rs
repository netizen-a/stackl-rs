use crate::ram;
use bitflags::bitflags;
use stackl::op;

#[allow(dead_code)]
pub struct MachineState {
    bp: i32,
    lp: i32,
    ip: i32,
    sp: i32,
    fp: i32,
    flag: MachineFlag,
    ivec: i32,
    vmem: i32,
    pub ram: ram::Memory,
}

impl MachineState {
    pub fn new(mem_size: i32) -> MachineState {
        MachineState {
            bp: 0,
            lp: mem_size,
            ip: 0,
            sp: 0,
            fp: 0,
            flag: MachineFlag::empty(),
            ivec: 0,
            vmem: 0,
            ram: ram::Memory::new(mem_size.try_into().unwrap()),
        }
    }

    pub fn set_sp(&mut self, addr: i32) {
        self.sp = addr;
    }
    pub fn run(mut self) {
        let sp_low = self.sp;
        loop {
            if self.flag.contains(MachineFlag::HALTED) {
                return;
            }
            assert!(sp_low <= self.sp);
            execute_inst(&mut self);
        }
    }
}

bitflags! {
    struct MachineFlag: i32 {
        const HALTED    = 0x01;
        const USER_MODE = 0x02;
        const INT_MODE  = 0x04;
        const INT_DIS   = 0x08;
        const VMEM      = 0x10;
    }
}

fn execute_inst(state: &mut MachineState) {
    let ram = &state.ram;
    let op: i32 = ram.load_i32(state.ip.try_into().unwrap()).unwrap();

    match op {
        op::NOP => {
            // println!("{:2}: nop ; {}", self.ip, op)
        }
        op::POP => {
            // println!("{:2}: pop", self.ip);
            state.sp -= 4;
        }
        op::PUSH => {
            state.ip += 4;
            let val = ram.load_i32(state.ip as _).unwrap();
            let result = ram.store_i32(val, state.sp.try_into().unwrap());
            assert!(result);
            // println!("{:2}: push {val}", state.ip);
            state.ip += 4;
            state.sp += 4;
            return;
        }
        op::PLUS => {
            // print!("{:2}: plus ; ", state.ip);
            state.sp -= 4;
            let lhs = ram.load_i32(state.sp as _).unwrap();
            let rhs = ram.load_i32((state.sp - 4) as _).unwrap();
            let result = lhs + rhs;
            let status = ram.store_i32(result, (state.sp - 4) as _);
            assert!(status);
            println!("{lhs} + {rhs} = {result}");
        }
        op::JMP => {
            // println!("{:2}: jmp", state.ip);
            state.ip += 4;
            state.ip = ram.load_i32(state.ip as _).unwrap();
            return;
        }
        op::JZ => {
            // println!("{:2}: jz", state.ip);
            state.sp -= 4;
            let val = ram.load_i32(state.sp as _).unwrap();
            if val == 0 {
                state.ip += 4;
                state.ip = ram.load_i32(state.ip as _).unwrap();
            } else {
                state.ip += 8;
            }
            return;
        }
        op::DUP => {
            let val = ram.load_i32(state.sp as _).unwrap();
            state.sp += 4;
            let result = ram.store_i32(val, state.sp as _);
            assert!(result);

            // println!("{:2}: dup ; sp = {}", state.ip, state.sp);
        }
        op::HALT => {
            // println!("{:2}: halt", state.ip);
            state.flag.set(MachineFlag::HALTED, true);
            return;
        }
        op::OUTS => {
            // println!("{:2}: outs", state.ip);
            let offset = ram.load_i32((state.sp - 4) as _).unwrap();
            let check = ram.print_str(offset as _);
            if let Err(check) = check {
                println!("{:?}", check);
                state.flag.set(MachineFlag::HALTED, true);
            }
        }
        k => unimplemented!("opcode {k}"),
    }
    state.ip += 4;
}
