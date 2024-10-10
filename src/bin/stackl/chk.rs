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
    #[allow(dead_code)]
    kind: CheckKind,
    #[allow(dead_code)]
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
        use CheckKind::*;
        match kind {
            IllegalInst => Self {
                kind,
                msg: "Illegal Instruction".to_string(),
            },
            IllegalAddr => Self {
                kind,
                msg: "Illegal Address".to_string(),
            },
            HwFailure => Self {
                kind,
                msg: "Hardware Failure".to_string(),
            },
            HwWarning => Self {
                kind,
                msg: "Hardware Warning".to_string(),
            },
            ProtInst => Self {
                kind,
                msg: "Protected Instruction".to_string(),
            },
            IllegalOp => Self {
                kind,
                msg: "Illegal Operation".to_string(),
            },
            _ => Self {
                kind: Other,
                msg: "Other".to_string(),
            },
        }
    }
}

impl fmt::Display for MachineCheck {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Machine Check: {}", self.msg)
    }
}
