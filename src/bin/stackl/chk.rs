use std::fmt;

#[derive(Debug)]
#[non_exhaustive]
pub enum CheckKind {
    /// Illegal Instruction
    IllegalInst = 0x00000001,
    /// Illegal Address
    IllegalAddr = 0x00000002,
    /// Hardware Failure
    #[allow(dead_code)]
    HwFailure = 0x00000004,
    /// Hardware Warning
    #[allow(dead_code)]
    HwWarning = 0x00000008,
    /// Protected Instruction
    ProtInst = 0x00000010,
    /// Illegal Operation
    IllegalOp = 0x00000020,
    Other,
}

#[derive(Debug)]
pub struct MachineCheck {
    kind: CheckKind,
    msg: String,
}

impl MachineCheck {
    pub fn new<E>(kind: CheckKind, msg: E) -> Self
    where
        E: ToString,
    {
        Self {
            kind,
            msg: msg.to_string(),
        }
    }
}

impl From<CheckKind> for MachineCheck {
    fn from(kind: CheckKind) -> Self {
        Self {
            kind,
            msg: String::new(),
        }
    }
}

impl fmt::Display for MachineCheck {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use CheckKind::*;
        let kind_msg = match self.kind {
            IllegalInst => "Illegal Instruction".to_string(),
            IllegalAddr => "Illegal Address".to_string(),
            HwFailure => "Hardware Failure".to_string(),
            HwWarning => "Hardware Warning".to_string(),
            ProtInst => "Protected Instruction".to_string(),
            IllegalOp => "Illegal Operation".to_string(),
            Other => "Other".to_string(),
        };
        write!(f, "Machine Check: {}{}", kind_msg, self.msg)
    }
}
