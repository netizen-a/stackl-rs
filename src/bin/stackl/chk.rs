#[derive(Debug)]
pub enum MachineCode {
    IllegalInst = 0x00000001,
    IllegalAddr = 0x00000002,
    #[allow(dead_code)]
    HwFailure = 0x00000004,
    #[allow(dead_code)]
    HwWarning = 0x00000008,
    ProtInst = 0x00000010,
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
