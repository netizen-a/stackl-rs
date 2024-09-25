#[derive(Debug)]
pub enum MachineCode {
    /// Illegal Instruction
    IllegalInst = 0x00000001,
    /// Illegal Address
    IllegalAddr = 0x00000002,
    /// Hardware Failure
    #[allow(dead_code)]
    HwFailure   = 0x00000004,
    /// Hardware Warning
    #[allow(dead_code)]
    HwWarning   = 0x00000008,
    /// Protected Instruction
    ProtInst    = 0x00000010,
    /// Illegal Operation
    IllegalOp   = 0x00000020,
}

#[derive(Debug)]
pub struct MachineCheck {
    #[allow(dead_code)]
    code: MachineCode,
    #[allow(dead_code)]
    msg: String,
}

impl MachineCheck {
    pub fn new<S: ToString>(code: MachineCode, msg: S) -> MachineCheck {
        MachineCheck {
            code,
            msg: msg.to_string(),
        }
    }
}
