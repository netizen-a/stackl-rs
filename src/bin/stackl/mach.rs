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
    fn replace_i32(&mut self, val: i32) -> Option<i32> {
        let offset = (self.sp - 4) as _;
        let tmp = self.ram.load_i32(offset)?;
        self.ram.store_i32(val, offset);
        Some(tmp)
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
        if value {
            eprintln!(
                "\n{:>8} {:>6} {:>6} {:>6} {:>6} {:>6}",
                "Flag", "BP", "LP", "IP", "SP", "FP"
            );
        }
    }
    pub fn run(mut self) {
        loop {
            if self.flag.contains(MachineFlag::HALTED) {
                return;
            }
            if self.flag.contains(MachineFlag::TRACE) {
                eprintln!(
                    "{:08x} {:6} {:6} {:6} {:6} {:6} {}",
                    self.flag.bits(),
                    self.bp,
                    self.lp,
                    self.ip,
                    self.sp,
                    self.fp,
                    self.ram.load_i32(self.ip as _).unwrap()
                );
            }
            execute_op(&mut self);
        }
    }
}

bitflags! {
    struct MachineFlag: i32 {
        const HALTED    = 0x00001;
        const USER_MODE = 0x00002;
        const INT_MODE  = 0x00004;
        const INT_DIS   = 0x00008;
        const VMEM      = 0x00010;
        const TRACE     = 0x00020;
        const I_MACH    = 0x10000;
        const I_TRAP    = 0x20000;
    }
}

fn execute_op(cpu: &mut MachineState) {
    let op: i32 = cpu.ram.load_i32(cpu.ip.try_into().unwrap()).unwrap();

    match op {
        op::NOP => {}
        op::ADD => {
            let rhs = cpu.pop_i32().unwrap();
            let lhs = cpu.pop_i32().unwrap();
            cpu.push_i32(lhs + rhs);
        }
        op::SUB => {
            let rhs = cpu.pop_i32().unwrap();
            let lhs = cpu.pop_i32().unwrap();
            cpu.push_i32(lhs - rhs);
        }
        op::MUL => {
            let rhs = cpu.pop_i32().unwrap();
            let lhs = cpu.pop_i32().unwrap();
            cpu.push_i32(lhs * rhs);
        }
        op::DIV => {
            let rhs = cpu.pop_i32().unwrap();
            let lhs = cpu.pop_i32().unwrap();
            if let Some(result) = lhs.checked_div(rhs) {
                cpu.push_i32(result);
            } else {
                println!("Machine Check: Div error");
                cpu.flag.set(MachineFlag::HALTED, true);
            }
        }
        op::MOD => {
            let rhs = cpu.pop_i32().unwrap();
            let lhs = cpu.pop_i32().unwrap();
            if let Some(result) = lhs.checked_rem_euclid(rhs) {
                cpu.push_i32(result);
            } else {
                println!("Machine Check: Mod error");
                cpu.flag.set(MachineFlag::HALTED, true);
            }
        }
        op::EQ => {
            let rhs = cpu.pop_i32().unwrap();
            let lhs = cpu.pop_i32().unwrap();
            cpu.push_i32((lhs == rhs) as i32);
        }
        op::NE => {
            let rhs = cpu.pop_i32().unwrap();
            let lhs = cpu.pop_i32().unwrap();
            cpu.push_i32((lhs != rhs) as i32);
        }
        op::GT => {
            let rhs = cpu.pop_i32().unwrap();
            let lhs = cpu.pop_i32().unwrap();
            cpu.push_i32((lhs > rhs) as i32);
        }
        op::LT => {
            let rhs = cpu.pop_i32().unwrap();
            let lhs = cpu.pop_i32().unwrap();
            cpu.push_i32((lhs < rhs) as i32);
        }
        op::GE => {
            let rhs = cpu.pop_i32().unwrap();
            let lhs = cpu.pop_i32().unwrap();
            cpu.push_i32((lhs >= rhs) as i32);
        }
        op::LE => {
            let rhs = cpu.pop_i32().unwrap();
            let lhs = cpu.pop_i32().unwrap();
            cpu.push_i32((lhs <= rhs) as i32);
        }
        op::AND => {
            let rhs = cpu.pop_i32().unwrap();
            let lhs = cpu.pop_i32().unwrap();
            cpu.push_i32((lhs != 0 && rhs != 0) as i32);
        }
        op::OR => {
            let rhs = cpu.pop_i32().unwrap();
            let lhs = cpu.pop_i32().unwrap();
            cpu.push_i32((lhs != 0 || rhs != 0) as i32);
        }
        op::NOT => {
            let val = cpu.pop_i32().unwrap();
            cpu.push_i32((val == 0) as i32);
        }
        op::SWAP => {
            let tmp = cpu.pop_i32().unwrap();
            let s1 = cpu.replace_i32(tmp).unwrap();
            cpu.push_i32(s1);
        }
        op::DUP => {
            let val = cpu.ram.load_i32((cpu.sp - 4) as _).unwrap();
            let result = cpu.ram.store_i32(val, cpu.sp as _);
            assert!(result);
            cpu.sp += 4;
        }
        op::HALT => {
            cpu.flag.set(MachineFlag::HALTED, true);
            return;
        }
        op::POP => {
            cpu.sp -= 4;
        }
        op::RETURN => {
            cpu.sp = cpu.fp - 4;
            cpu.ip = cpu.ram.load_i32((cpu.fp - 8) as _).unwrap();
            cpu.fp = cpu.ram.load_i32((cpu.fp - 4) as _).unwrap();
            return;
        }
        op::RETURNV => {
            let tmp = cpu.ram.load_i32((cpu.sp - 4) as _).unwrap();
            cpu.sp = cpu.fp - 4;
            cpu.ip = cpu.ram.load_i32((cpu.fp - 8) as _).unwrap();
            cpu.fp = cpu.ram.load_i32((cpu.fp - 4) as _).unwrap();
            cpu.ram.store_i32(tmp, (cpu.sp - 4) as _);
            return;
        }
        op::NEG => {
            let val = cpu.pop_i32().unwrap();
            cpu.push_i32(-val);
        }
        op::PUSHCVARIND => {
            let offset = cpu.pop_i32().unwrap();
            let val = cpu.ram.load_u8(offset as _).unwrap();
            cpu.push_i32(val as _);
        }
        op::OUTS => {
            let offset = cpu.ram.load_i32((cpu.sp - 4) as _).unwrap();
            let check = cpu.ram.print_str(offset as _);
            if let Err(check) = check {
                println!("{:?}", check);
                cpu.flag.set(MachineFlag::HALTED, true);
            }
        }
        op::INP => {
            unimplemented!("inp");
        }
        op::PUSHFP => {
            cpu.push_i32(cpu.fp);
        }
        op::JMPUSER => {
            cpu.ip += 4;
            cpu.ip = cpu.ram.load_i32(cpu.ip as _).unwrap();
            cpu.flag.set(MachineFlag::USER_MODE, true);
        }
        op::TRAP => {
            unimplemented!("trap");
        }
        op::RTI => {
            unimplemented!("rti");
        }
        op::CALLI => {
            let tmp = cpu.pop_i32().unwrap();
            cpu.push_i32(cpu.ip + 4);
            cpu.push_i32(cpu.fp);
            cpu.fp = cpu.sp;
            cpu.ip = tmp;
            return;
        }
        op::PUSHREG => {
            cpu.ip += 4;
            let reg = cpu.ram.load_i32(cpu.ip as _).unwrap();
            match reg {
                0 => cpu.push_i32(cpu.bp),
                1 => cpu.push_i32(cpu.lp),
                2 => cpu.push_i32(cpu.ip),
                3 => cpu.push_i32(cpu.sp),
                4 => cpu.push_i32(cpu.fp),
                5 => cpu.push_i32(cpu.flag.bits()),
                _ => panic!("Machine check"),
            }
        }
        op::POPREG => {
            cpu.ip += 4;
            let reg = cpu.ram.load_i32(cpu.ip as _).unwrap();
            match reg {
                0 => cpu.bp = cpu.pop_i32().unwrap(),
                1 => cpu.lp = cpu.pop_i32().unwrap(),
                2 => {
                    cpu.ip = cpu.pop_i32().unwrap();
                    return;
                }
                3 => cpu.sp = cpu.pop_i32().unwrap(),
                4 => cpu.fp = cpu.pop_i32().unwrap(),
                5 => cpu.flag = MachineFlag::from_bits(cpu.pop_i32().unwrap()).unwrap(),
                _ => panic!("Machine check"),
            }
        }
        op::BAND => {
            let rhs = cpu.pop_i32().unwrap();
            let lhs = cpu.pop_i32().unwrap();
            cpu.push_i32(lhs & rhs);
        }
        op::BOR => {
            let rhs = cpu.pop_i32().unwrap();
            let lhs = cpu.pop_i32().unwrap();
            cpu.push_i32(lhs | rhs);
        }
        op::BXOR => {
            let rhs = cpu.pop_i32().unwrap();
            let lhs = cpu.pop_i32().unwrap();
            cpu.push_i32(lhs ^ rhs);
        }
        op::SHIFTL => {
            let rhs = cpu.pop_i32().unwrap();
            let lhs = cpu.pop_i32().unwrap();
            cpu.push_i32(lhs << rhs);
        }
        op::SHIFTR => {
            let rhs = cpu.pop_i32().unwrap();
            let lhs = cpu.pop_i32().unwrap();
            cpu.push_i32(lhs >> rhs);
        }
        op::PUSHVARIND => {
            let offset = cpu.pop_i32().unwrap();
            let val = cpu.ram.load_i32(offset as _).unwrap();
            cpu.push_i32(val);
        }
        op::POPCVARIND => {
            let offset = cpu.pop_i32().unwrap();
            let val = cpu.pop_i32().unwrap();
            cpu.ram.store_u8(val as u8, offset as _);
        }
        op::POPVARIND => {
            let offset = cpu.pop_i32().unwrap();
            let val = cpu.pop_i32().unwrap();
            cpu.ram.store_i32(val, offset as _);
        }
        op::COMP => {
            let val = cpu.pop_i32().unwrap();
            cpu.push_i32(!val);
        }
        op::PUSH => {
            cpu.ip += 4;
            let val = cpu.ram.load_i32(cpu.ip as _).unwrap();
            cpu.push_i32(val);
        }
        op::JMP => {
            cpu.ip += 4;
            cpu.ip = cpu.ram.load_i32(cpu.ip as _).unwrap();
            return;
        }
        op::JZ => {
            let val = cpu.pop_i32().unwrap();
            if val == 0 {
                cpu.ip += 4;
                cpu.ip = cpu.ram.load_i32(cpu.ip as _).unwrap();
            } else {
                cpu.ip += 8;
            }
            return;
        }
        op::PUSHVAR => {
            cpu.ip += 4;
            let offset = cpu.ram.load_i32(cpu.ip as _).unwrap();
            let val = cpu.ram.load_i32((cpu.fp + offset) as _).unwrap();
            cpu.push_i32(val);
        }
        op::POPVAR => {
            cpu.ip += 4;
            let offset = cpu.ram.load_i32(cpu.ip as _).unwrap();
            let val = cpu.pop_i32().unwrap();
            let result = cpu.ram.store_i32(val, (cpu.fp + offset) as _);
            assert!(result);
        }
        op::ADJSP => {
            cpu.ip += 4;
            cpu.sp += cpu.ram.load_i32(cpu.ip as _).unwrap();
        }
        op::POPARGS => {
            let tmp = cpu.pop_i32().unwrap();
            cpu.ip += 4;
            cpu.sp -= cpu.ram.load_i32(cpu.ip as _).unwrap();
            cpu.push_i32(tmp);
        }
        op::CALL => {
            cpu.push_i32(cpu.ip + 8);
            cpu.push_i32(cpu.fp);
            cpu.fp = cpu.sp;
            cpu.ip = cpu.ram.load_i32((cpu.ip + 4) as _).unwrap();
            return;
        }
        op::PUSHCVAR => {
            cpu.ip += 4;
            let offset = cpu.ram.load_i32(cpu.ip as _).unwrap();
            let val = cpu.ram.load_u8((cpu.fp + offset) as _).unwrap();
            cpu.push_i32(val.into());
        }
        op::POPCVAR => {
            cpu.ip += 4;
            let offset = cpu.ram.load_i32(cpu.ip as _).unwrap();
            let val = cpu.pop_i32().unwrap();
            let result = cpu.ram.store_u8(val as _, (cpu.fp + offset) as _);
            assert!(result);
        }
        op::SET_TRACE => {
            cpu.set_trace(true);
        }
        op::CLR_TRACE => {
            cpu.set_trace(false);
        }
        op::CLR_INT_DIS => {
            cpu.flag.set(MachineFlag::INT_DIS, false);
        }
        op::SET_INT_DIS => {
            cpu.flag.set(MachineFlag::INT_DIS, true);
        }
        55..=i32::MAX | i32::MIN..0 => {
            unimplemented!("illegal");
        }
    }
    cpu.ip += 4;
}
