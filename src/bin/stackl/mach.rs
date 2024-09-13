use crate::ram;
use stackl::op;

#[allow(dead_code)]
pub struct MachineState {
    bp: i32,
    lp: i32,
    ip: i32,
    sp: i32,
    fp: i32,
    flag: i32,
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
            flag: 0,
            ivec: 0,
            vmem: 0,
            ram: ram::Memory::new(mem_size.try_into().unwrap()),
        }
    }

    pub fn set_sp(&mut self, addr: i32) {
        self.sp = addr;
    }
    pub fn execute(mut self) {
        let sp_low = self.sp;
        loop {
            assert!(sp_low <= self.sp);
            let op: i16 = self.ram.load_i16(self.ip.try_into().unwrap()).unwrap();

            match op {
                op::NOP => {println!("{:2}: nop ; {}", self.ip, op)}
                op::POP => {
                    println!("{:2}: pop", self.ip);
                    self.sp -= 4;
                }
                op::PUSH => {
                    let operand = 2 + self.ip;
                    let val = self.ram.load_i32(operand.try_into().unwrap()).unwrap();
                    let result=self.ram.store_i32(val, self.sp.try_into().unwrap());
                    assert!(result);

                    println!("{:2}: push {val}", self.ip);

                    self.ip += 6;
                    self.sp += 4;
                    continue;
                }
                op::PLUS => {
                    print!("{:2}: plus ; ", self.ip);
                    self.sp -= 4;
                    let lhs = self.ram.load_i32(self.sp.try_into().unwrap()).unwrap();
                    self.sp -= 4;
                    let rhs = self.ram.load_i32(self.sp.try_into().unwrap()).unwrap();
                    let result = lhs + rhs;
                    let status = self.ram.store_i32(result, self.sp.try_into().unwrap());
                    assert!(status);
                    self.sp += 4;
                    println!("{lhs} + {rhs} = {result}");
                }
                op::JMP => {
                    println!("{:2}: jmp", self.ip);
                    self.ip += 2;
                    self.ip = self.ram.load_i32(self.ip.try_into().unwrap()).unwrap();
                    continue;
                }
                op::JZ => {
                    println!("{:2}: jz", self.ip);
                    self.sp -= 4;
                    let val = self.ram.load_i32(self.sp.try_into().unwrap()).unwrap();
                    let operand = 2 + self.ip;
                    if val == 0 {
                        self.ip = self.ram.load_i32(operand.try_into().unwrap()).unwrap();
                    }
                    continue;
                }
                op::HALT => {
                    println!("{:2}: halt", self.ip);
                    return;
                }
                k => unimplemented!("opcode {k}"),
            }
            self.ip += 2;
            // println!("next ip: {}", self.ip);
        }
    }
}
