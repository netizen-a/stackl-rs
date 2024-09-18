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
    fn push_i32(&mut self, val: i32) {
        let result = self.ram.store_i32(val, self.sp as _);
        assert!(result);
        self.sp += 4;
    }
    fn pop_i32(&mut self) -> Option<i32> {
        self.sp -= 4;
        self.ram.load_i32(self.sp as _)
    }
    pub fn set_sp(&mut self, addr: i32) {
        self.sp = addr;
    }
    pub fn set_trace(&mut self, value: bool) {
        self.flag.set(MachineFlag::TRACE, value);
    }
    pub fn run(mut self) {
        loop {
            if self.flag.contains(MachineFlag::TRACE) {
                eprintln!("{:08x} {:6} {:6} {:6} {:6} {:6} {}",
                    self.flag.bits(), self.bp, self.lp, self.ip, self.sp, self.fp,
                    self.ram.load_i32(self.ip as _).unwrap()
                );
            }
            if self.flag.contains(MachineFlag::HALTED) {
                return;
            }
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
        const TRACE    = 0x20;
    }
}

fn execute_inst(state: &mut MachineState) {
    let op: i32 = state.ram.load_i32(state.ip.try_into().unwrap()).unwrap();

    match op {
        op::NOP => {}
        op::ADD => {
            let rhs = state.pop_i32().unwrap();
            let lhs = state.pop_i32().unwrap();
            state.push_i32(lhs + rhs);
        }
        op::SUB => {
            let rhs = state.pop_i32().unwrap();
            let lhs = state.pop_i32().unwrap();
            state.push_i32(lhs - rhs);
        }
        op::MUL => {
            let rhs = state.pop_i32().unwrap();
            let lhs = state.pop_i32().unwrap();
            state.push_i32(lhs * rhs);
        }
        op::DIV => {
            let rhs = state.pop_i32().unwrap();
            let lhs = state.pop_i32().unwrap();
            if let Some(result) = lhs.checked_div(rhs) {
                state.push_i32(result);
            } else {
                println!("Machine Check: Div error");
                state.flag.set(MachineFlag::HALTED, true);
            }
        }
        op::MOD => {
            let rhs = state.pop_i32().unwrap();
            let lhs = state.pop_i32().unwrap();
            if let Some(result) = lhs.checked_rem_euclid(rhs) {
                state.push_i32(result);
            } else {
                println!("Machine Check: Mod error");
                state.flag.set(MachineFlag::HALTED, true);
            }
        }
        op::EQ => {
            let rhs = state.pop_i32().unwrap();
            let lhs = state.pop_i32().unwrap();
            state.push_i32((lhs == rhs) as i32);
        }
        op::NE => {
            let rhs = state.pop_i32().unwrap();
            let lhs = state.pop_i32().unwrap();
            state.push_i32((lhs != rhs) as i32);
        }
        op::GT => {
            let rhs = state.pop_i32().unwrap();
            let lhs = state.pop_i32().unwrap();
            state.push_i32((lhs > rhs) as i32);
        }
        op::LT => {
            let rhs = state.pop_i32().unwrap();
            let lhs = state.pop_i32().unwrap();
            state.push_i32((lhs < rhs) as i32);
        }
        op::GE => {
            let rhs = state.pop_i32().unwrap();
            let lhs = state.pop_i32().unwrap();
            state.push_i32((lhs >= rhs) as i32);
        }
        op::LE => {
            let rhs = state.pop_i32().unwrap();
            let lhs = state.pop_i32().unwrap();
            state.push_i32((lhs <= rhs) as i32);
        }
        op::AND => {
            let rhs = state.pop_i32().unwrap();
            let lhs = state.pop_i32().unwrap();
            state.push_i32((lhs != 0 && rhs != 0) as i32);
        }
        op::OR => {
            let rhs = state.pop_i32().unwrap();
            let lhs = state.pop_i32().unwrap();
            state.push_i32((lhs != 0 || rhs != 0) as i32);
        }
        op::NOT => {
            let val = state.pop_i32().unwrap();
            state.push_i32((!(val != 0)) as i32);
        }
        op::DUP => {
            let val = state.ram.load_i32((state.sp - 4) as _).unwrap();
            let result = state.ram.store_i32(val, state.sp as _);
            assert!(result);
            // println!("{:2}: dup {val} ; sp = {}", state.ip, state.sp);
            state.sp += 4;
        }
        op::HALT => {
            // println!("{:2}: halt", state.ip);
            state.flag.set(MachineFlag::HALTED, true);
            return;
        }
        op::POP => {
            // println!("{:2}: pop", self.ip);
            state.sp -= 4;
        }
        op::NEG => {
            let val = state.pop_i32().unwrap();
            state.push_i32(-val);
        }
        op::OUTS => {
            // println!("{:2}: outs", state.ip);
            let offset = state.ram.load_i32((state.sp - 4) as _).unwrap();
            let check = state.ram.print_str(offset as _);
            if let Err(check) = check {
                println!("{:?}", check);
                state.flag.set(MachineFlag::HALTED, true);
            }
        }
        op::JMPUSER => {
            state.ip += 4;
            state.ip = state.ram.load_i32(state.ip as _).unwrap();
            state.flag.set(MachineFlag::USER_MODE, true);
        }
        op::PUSHREG => {
            state.ip += 4;
            let reg = state.ram.load_i32(state.ip as _).unwrap();
            match reg {
                0 => state.push_i32(state.bp),
                1 => state.push_i32(state.lp),
                2 => state.push_i32(state.ip),
                3 => state.push_i32(state.sp),
                4 => state.push_i32(state.fp),
                5 => state.push_i32(state.flag.bits()),
                _ => panic!("Machine check"),
            }
        }
        op::POPREG => {
            state.ip += 4;
            let reg = state.ram.load_i32(state.ip as _).unwrap();
            match reg {
                0 => state.bp = state.pop_i32().unwrap(),
                1 => state.lp = state.pop_i32().unwrap(),
                2 => {
                    state.ip = state.pop_i32().unwrap();
                    return;
                }
                3 => state.sp = state.pop_i32().unwrap(),
                4 => state.fp = state.pop_i32().unwrap(),
                5 => state.flag = MachineFlag::from_bits(state.pop_i32().unwrap()).unwrap(),
                _ => panic!("Machine check"),
            }
        }
        op::BAND => {
            let rhs = state.pop_i32().unwrap();
            let lhs = state.pop_i32().unwrap();
            state.push_i32(lhs & rhs);
        }
        op::BOR => {
            let rhs = state.pop_i32().unwrap();
            let lhs = state.pop_i32().unwrap();
            state.push_i32(lhs | rhs);
        }
        op::BXOR => {
            let rhs = state.pop_i32().unwrap();
            let lhs = state.pop_i32().unwrap();
            state.push_i32(lhs ^ rhs);
        }
        op::SHIFTL => {
            let rhs = state.pop_i32().unwrap();
            let lhs = state.pop_i32().unwrap();
            state.push_i32(lhs << rhs);
        }
        op::SHIFTR => {
            let rhs = state.pop_i32().unwrap();
            let lhs = state.pop_i32().unwrap();
            state.push_i32(lhs >> rhs);
        }
        op::COMP => {
            let val = state.pop_i32().unwrap();
            state.push_i32(!val);
        }
        op::PUSH => {
            state.ip += 4;
            let val = state.ram.load_i32(state.ip as _).unwrap();
            state.push_i32(val);
        }
        op::JMP => {
            // println!("{:2}: jmp", state.ip);
            state.ip += 4;
            state.ip = state.ram.load_i32(state.ip as _).unwrap();
            return;
        }
        op::JZ => {
            let val = state.pop_i32().unwrap();
            if val == 0 {
                state.ip += 4;
                state.ip = state.ram.load_i32(state.ip as _).unwrap();
            } else {
                state.ip += 8;
            }
            return;
        }
        op::TRACEON => {
            eprintln!("\n{:>8} {:>6} {:>6} {:>6} {:>6} {:>6}",
                "Flag", "BP", "LP", "IP", "SP", "FP");
            state.set_trace(true);
        }
        op::TRACEOFF => {
            state.set_trace(false);
        }
        k => unimplemented!("opcode {k}"),
    }
    state.ip += 4;
}
