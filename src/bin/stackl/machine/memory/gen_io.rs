pub const GEN_IO_CSR: i32 = 0x0C000000; // Control/Status register
pub const GEN_IO_BUFF: i32 = 0x0C000004; // Buffer address register
pub const GEN_IO_SIZE: i32 = 0x0C000008; // Buffer size register
pub const GEN_IO_COUNT: i32 = 0x0C00000C; // Number of characters sent

pub const GEN_IO_CSR_IE: i32 = 0x00010000; // Interrupt enable
pub const GEN_IO_CSR_INT: i32 = 0x00020000; // Interrupt occurred
pub const GEN_IO_CSR_DONE: i32 = 0x80000000u32 as i32; // The operation is complete
pub const GEN_IO_CSR_ERR: i32 = 0x40000000; // The operation resulted in an error

// Operations: occupy the lower 8 bits of the CSR register
pub const GEN_IO_OP_PRINTS: i32 = 1;
pub const GEN_IO_OP_PRINTC: i32 = 2;
pub const GEN_IO_OP_GETL: i32 = 3;
pub const GEN_IO_OP_GETI: i32 = 4;
pub const GEN_IO_OP_EXEC: i32 = 5;
