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
    pub fn run(mut self) {
        loop {
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
    }
}

fn execute_inst(state: &mut MachineState) {
    let ram = &state.ram;
    let op: i32 = ram.load_i32(state.ip.try_into().unwrap()).unwrap();

    match op {
        op::NOP => {
            // println!("{:2}: nop ; {}", self.ip, op)
        }
        op::ADD => {
            // print!("{:2}: add ; ", state.ip);
            state.sp -= 4;
            let lhs = ram.load_i32((state.sp - 4) as _).unwrap();
            let rhs = ram.load_i32(state.sp as _).unwrap();
            let result = lhs + rhs;
            let status = ram.store_i32(result, (state.sp - 4) as _);
            assert!(status);
            println!("{lhs} + {rhs} = {result}");
        }
        op::SUB => {
            let rhs = state.pop_i32().unwrap();
            let lhs = state.pop_i32().unwrap();
            state.push_i32(lhs - rhs);
        }
        op::MUL => {
            state.sp -= 4;
            let lhs = ram.load_i32((state.sp - 4) as _).unwrap();
            let rhs = ram.load_i32(state.sp as _).unwrap();
            let result = lhs * rhs;
            let status = ram.store_i32(result, (state.sp - 4) as _);
            assert!(status);
            println!("{lhs} * {rhs} = {result}");
        }
        op::DIV => {
            state.sp -= 4;
            let lhs = ram.load_i32((state.sp - 4) as _).unwrap();
            let rhs = ram.load_i32(state.sp as _).unwrap();
            if let Some(result) = lhs.checked_div(rhs) {
                let status = ram.store_i32(result, (state.sp - 4) as _);
                assert!(status);
                println!("{lhs} / {rhs} = {result}");
            } else {
                println!("Machine Check: Div error");
                state.flag.set(MachineFlag::HALTED, true);
            }
        }
        op::MOD => {
            state.sp -= 4;
            let lhs = ram.load_i32((state.sp - 4) as _).unwrap();
            let rhs = ram.load_i32(state.sp as _).unwrap();
            if let Some(result) = lhs.checked_rem_euclid(rhs) {
                let status = ram.store_i32(result, (state.sp - 4) as _);
                assert!(status);
                println!("{lhs} % {rhs} = {result}");
            } else {
                println!("Machine Check: Mod error");
                state.flag.set(MachineFlag::HALTED, true);
            }
        }
        op::EQ => {
            state.sp -= 4;
            let lhs = ram.load_i32((state.sp - 4) as _).unwrap();
            let rhs = ram.load_i32(state.sp as _).unwrap();
            let result = (lhs == rhs) as i32;
            let status = ram.store_i32(result, (state.sp - 4) as _);
            assert!(status);
            println!("{lhs} == {rhs} = {result}");
        }
        op::NE => {
            state.sp -= 4;
            let lhs = ram.load_i32((state.sp - 4) as _).unwrap();
            let rhs = ram.load_i32(state.sp as _).unwrap();
            let result = (lhs != rhs) as i32;
            let status = ram.store_i32(result, (state.sp - 4) as _);
            assert!(status);
            println!("{lhs} != {rhs} = {result}");
        }
        op::GT => {
            state.sp -= 4;
            let lhs = ram.load_i32((state.sp - 4) as _).unwrap();
            let rhs = ram.load_i32(state.sp as _).unwrap();
            let result = (lhs > rhs) as i32;
            let status = ram.store_i32(result, (state.sp - 4) as _);
            assert!(status);
            println!("{lhs} > {rhs} = {result}");
        }
        op::LT => {
            state.sp -= 4;
            let lhs = ram.load_i32((state.sp - 4) as _).unwrap();
            let rhs = ram.load_i32(state.sp as _).unwrap();
            let result = (lhs < rhs) as i32;
            let status = ram.store_i32(result, (state.sp - 4) as _);
            assert!(status);
            println!("{lhs} < {rhs} = {result}");
        }
        op::GE => {
            state.sp -= 4;
            let lhs = ram.load_i32((state.sp - 4) as _).unwrap();
            let rhs = ram.load_i32(state.sp as _).unwrap();
            let result = (lhs >= rhs) as i32;
            let status = ram.store_i32(result, (state.sp - 4) as _);
            assert!(status);
            println!("{lhs} >= {rhs} = {result}");
        }
        op::LE => {
            state.sp -= 4;
            let lhs = ram.load_i32((state.sp - 4) as _).unwrap();
            let rhs = ram.load_i32(state.sp as _).unwrap();
            let result = (lhs <= rhs) as i32;
            let status = ram.store_i32(result, (state.sp - 4) as _);
            assert!(status);
            println!("{lhs} <= {rhs} = {result}");
        }
        op::AND => {
            state.sp -= 4;
            let lhs = ram.load_i32((state.sp - 4) as _).unwrap();
            let rhs = ram.load_i32(state.sp as _).unwrap();
            let result = (lhs != 0 && rhs != 0) as i32;
            let status = ram.store_i32(result, (state.sp - 4) as _);
            assert!(status);
            println!("{lhs} /\\ {rhs} = {result}");
        }
        op::OR => {
            state.sp -= 4;
            let lhs = ram.load_i32((state.sp - 4) as _).unwrap();
            let rhs = ram.load_i32(state.sp as _).unwrap();
            let result = (lhs != 0 || rhs != 0) as i32;
            let status = ram.store_i32(result, (state.sp - 4) as _);
            assert!(status);
            println!("{lhs} \\/ {rhs} = {result}");
        }
        op::NOT => {
            state.sp -= 4;
            let val = ram.load_i32(state.sp as _).unwrap();
            let result = (!(val != 0)) as i32;
            let status = ram.store_i32(result, (state.sp - 4) as _);
            assert!(status);
            println!("!{val} = {result}");
        }
        op::DUP => {
            let val = ram.load_i32((state.sp - 4) as _).unwrap();
            let result = ram.store_i32(val, state.sp as _);
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
        op::OUTS => {
            // println!("{:2}: outs", state.ip);
            let offset = ram.load_i32((state.sp - 4) as _).unwrap();
            let check = ram.print_str(offset as _);
            if let Err(check) = check {
                println!("{:?}", check);
                state.flag.set(MachineFlag::HALTED, true);
            }
        }
        op::JMPUSER => {
            state.ip += 4;
            state.ip = ram.load_i32(state.ip as _).unwrap();
            state.flag.set(MachineFlag::USER_MODE, true);
        }
        op::PUSHREG => {
            let reg = ram.load_i32((state.ip + 4) as _).unwrap();
            let status = match reg {
                0 => ram.store_i32(state.bp, state.sp as _),
                1 => ram.store_i32(state.lp, state.sp as _),
                2 => ram.store_i32(state.ip, state.sp as _),
                3 => ram.store_i32(state.sp, state.sp as _),
                4 => ram.store_i32(state.fp, state.sp as _),
                5 => ram.store_i32(state.flag.bits(), state.sp as _),
                _ => panic!("Machine check"),
            };
            assert!(status);
            state.sp += 4;
            state.ip += 4;
        }
        op::PUSH => {
            state.ip += 4;
            let val = ram.load_i32(state.ip as _).unwrap();
            state.push_i32(val);
            // println!("{:2}: push {val}", state.ip);
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
        k => unimplemented!("opcode {k}"),
    }
    state.ip += 4;
}
