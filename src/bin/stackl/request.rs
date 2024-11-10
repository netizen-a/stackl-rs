use super::*;

#[derive(Debug, Clone)]
pub struct Request {
    pub offset: i32,
    pub op: i32,
    pub param1: i32,
    pub param2: i32,
    pub bp: i32,
}

pub fn process_request(machine: &RwLock<MachineState>, request: &Request) -> Result<(), MachineCheck> {
    const INP_PRINTS_CALL: i32 = 3;
    const INP_GETS_CALL: i32 = 5;
    const INP_GETL_CALL: i32 = 6;
    const INP_GETI_CALL: i32 = 7;
    const INP_EXEC_CALL: i32 = 8;
    let op = request.op;
    let param1 = request.param1;
    let _param2 = request.param2;
    match op {
        INP_PRINTS_CALL => {
            let read_lock = machine.read().unwrap();
            read_lock.print(param1)
        }
        INP_GETS_CALL => {
            let mut buf = String::new();
            io::stdin().read_line(&mut buf).unwrap();
            let mut write_lock = machine.write().unwrap();
            write_lock.store_slice(buf.as_bytes(), param1)
        }
        INP_GETL_CALL => {
            let mut buf = String::new();
            io::stdin().read_line(&mut buf).unwrap();
            buf.truncate(255);
            buf.push('\0');
            let mut write_lock = machine.write().unwrap();
            write_lock.store_slice(buf.as_bytes(), param1)
        }
        INP_GETI_CALL => {
            let mut buf = String::new();
            io::stdin().read_line(&mut buf).unwrap();
            let Ok(deci) = i32::from_str(buf.trim()) else {
                return Err(flag::MachineCheck::ILLEGAL_INST);
            };
            let mut write_lock = machine.write().unwrap();
            write_lock.store_i32(deci, param1)
        }
        INP_EXEC_CALL => {
            let read_lock = machine.read().unwrap();
            let c_str = read_lock.load_cstr(param1)?;
            let Ok(filepath) = c_str.to_str() else {
                return Err(MachineCheck::ILLEGAL_INST);
            };
            let Ok(content) = fs::read(filepath) else {
                return Err(MachineCheck::ILLEGAL_INST);
            };
            drop(read_lock);

            let program = match StacklFormatV2::try_from(content.as_slice()) {
                Ok(data) => data,
                Err(stackl::ErrorKind::InvalidMagic) => {
                    let fmt1 = StacklFormatV1::try_from(content.as_slice()).unwrap();
                    fmt1.try_into().unwrap()
                }
                Err(err) => {
                    panic!("failed to load: {:?}", err);
                }
            };
            let mut machine_lock = machine.write().unwrap();
            let high_mem = program.text.len() as i32 + request.bp;
            machine_lock.store_i32(high_mem, request.offset + 8)?;
            machine_lock.store_program(program, false, request.bp)
        }
        _ => Err(MachineCheck::ILLEGAL_INST),
    }
}
