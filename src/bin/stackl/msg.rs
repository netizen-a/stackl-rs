use crate::chk::MachineCheck;

#[derive(Debug)]
pub enum MachineResponse {
    Ok,
    #[allow(dead_code)]
    Err(MachineCheck),
}

#[derive(Debug)]
pub enum MachineRequest {
    Unknown,
    Prints(i32),
}
