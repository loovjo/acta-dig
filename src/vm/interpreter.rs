use std::convert::TryInto;

use super::types::*;
use crate::messages::{ActorAddr, Atom, Value};

pub struct VMState<'a> {
    actor_code: &'a [u8],
    instruction_pointer: usize,
    program_end: usize,

    pub registers: [Value; 256],
}

impl<'a> VMState<'a> {
    pub fn new(actor_code: &'a [u8], (start, end): (usize, usize)) -> VMState {
        assert!(start <= end);
        assert!(end <= actor_code.len());
        let registers = {
            let mut registers: [std::mem::MaybeUninit<Value>; 256] =
                unsafe { std::mem::MaybeUninit::uninit().assume_init() };

            for elem in &mut registers[..] {
                unsafe {
                    std::ptr::write(elem.as_mut_ptr(), Value::Number(0));
                }
            }

            unsafe { std::mem::transmute::<_, [Value; 256]>(registers) }
        };

        VMState {
            actor_code,
            instruction_pointer: start,
            program_end: end,
            registers,
        }
    }

    pub fn read_u8(&mut self) -> Result<u8, VMError> {
        if self.instruction_pointer >= self.program_end {
            return Err(VMError::OutOfBounds);
        }
        let value = self.actor_code[self.instruction_pointer];
        self.instruction_pointer += 1;

        Ok(value)
    }

    fn read_u64(&mut self) -> Result<u64, VMError> {
        if self.instruction_pointer + 8 > self.program_end {
            return Err(VMError::OutOfBounds);
        }
        let bytes = &self.actor_code[self.instruction_pointer..self.instruction_pointer + 8];
        self.instruction_pointer += 8;

        let value = u64::from_le_bytes(bytes.try_into().unwrap());

        Ok(value)
    }

    fn read_f64(&mut self) -> Result<f64, VMError> {
        if self.instruction_pointer + 8 > self.program_end {
            return Err(VMError::OutOfBounds);
        }
        let bytes = &self.actor_code[self.instruction_pointer..self.instruction_pointer + 8];
        self.instruction_pointer += 8;

        let value = f64::from_le_bytes(bytes.try_into().unwrap());

        Ok(value)
    }

    pub fn step_once(&mut self, io_state: &IOState) -> Result<Option<IO>, VMError> {
        let inst = match self.read_u8() {
            Ok(inst) => inst,
            Err(VMError::OutOfBounds) => return Ok(Some(IO::Done)),
            Err(e) => return Err(e),
        };
        match inst {
            0x00 => {
                // set_self_addr
                let reg = self.read_u8()?;
                self.registers[reg as usize] = Value::ActorAddr(io_state.self_addr);

                Ok(None)
            }
            0x01 => {
                // set_float
                let reg = self.read_u8()?;
                let value = self.read_f64()?;
                self.registers[reg as usize] = Value::Float(value);

                Ok(None)
            }
            0x02 => {
                // set_integer
                let reg = self.read_u8()?;
                let value = self.read_u64()?;
                self.registers[reg as usize] = Value::Number(value);

                Ok(None)
            }
            // TODO: 0x03/set_string
            0x04 => {
                // copy
                let dest = self.read_u8()?;
                let source = self.read_u8()?;
                self.registers[dest as usize] = self.registers[source as usize].clone();

                Ok(None)
            }
            0x05 => {
                // generate_atom
                let reg = self.read_u8()?;

                self.registers[reg as usize] = Value::Atom(Atom::random());

                Ok(None)
            }
            0x06 => {
                // integer_to_atom
                let reg = self.read_u8()?;

                match self.registers[reg as usize] {
                    Value::Number(n) => {
                        self.registers[reg as usize] = Value::Atom(Atom(n));

                        Ok(None)
                    }
                    ref val => Err(VMError::WrongValueType(val.clone())),
                }
            }
            0x10 => {
                // add_int
                let dest = self.read_u8()?;
                let source = self.read_u8()?;

                match self.registers[source as usize] {
                    Value::Number(source_val) => match &mut self.registers[dest as usize] {
                        &mut Value::Number(ref mut dest_val) => {
                            *dest_val += source_val;
                            Ok(None)
                        }
                        val => Err(VMError::WrongValueType(val.clone())),
                    },
                    ref val => Err(VMError::WrongValueType(val.clone())),
                }
            }
            0x11 => {
                // sub_int
                let dest = self.read_u8()?;
                let source = self.read_u8()?;

                match self.registers[source as usize] {
                    Value::Number(source_val) => match &mut self.registers[dest as usize] {
                        &mut Value::Number(ref mut dest_val) => {
                            *dest_val -= source_val;
                            Ok(None)
                        }
                        val => Err(VMError::WrongValueType(val.clone())),
                    },
                    ref val => Err(VMError::WrongValueType(val.clone())),
                }
            }
            0x12 => {
                // mul_int
                let dest = self.read_u8()?;
                let source = self.read_u8()?;

                match self.registers[source as usize] {
                    Value::Number(source_val) => match &mut self.registers[dest as usize] {
                        &mut Value::Number(ref mut dest_val) => {
                            *dest_val *= source_val;
                            Ok(None)
                        }
                        val => Err(VMError::WrongValueType(val.clone())),
                    },
                    ref val => Err(VMError::WrongValueType(val.clone())),
                }
            }
            0x20 => {
                // add_float
                let dest = self.read_u8()?;
                let source = self.read_u8()?;

                match self.registers[source as usize] {
                    Value::Float(source_val) => match &mut self.registers[dest as usize] {
                        &mut Value::Float(ref mut dest_val) => {
                            *dest_val += source_val;
                            Ok(None)
                        }
                        val => Err(VMError::WrongValueType(val.clone())),
                    },
                    ref val => Err(VMError::WrongValueType(val.clone())),
                }
            }
            0x21 => {
                // sub_float
                let dest = self.read_u8()?;
                let source = self.read_u8()?;

                match self.registers[source as usize] {
                    Value::Float(source_val) => match &mut self.registers[dest as usize] {
                        &mut Value::Float(ref mut dest_val) => {
                            *dest_val -= source_val;
                            Ok(None)
                        }
                        val => Err(VMError::WrongValueType(val.clone())),
                    },
                    ref val => Err(VMError::WrongValueType(val.clone())),
                }
            }
            0x22 => {
                // mul_float
                let dest = self.read_u8()?;
                let source = self.read_u8()?;

                match self.registers[source as usize] {
                    Value::Float(source_val) => match &mut self.registers[dest as usize] {
                        &mut Value::Float(ref mut dest_val) => {
                            *dest_val *= source_val;
                            Ok(None)
                        }
                        val => Err(VMError::WrongValueType(val.clone())),
                    },
                    ref val => Err(VMError::WrongValueType(val.clone())),
                }
            }
            0x23 => {
                // div_float
                let dest = self.read_u8()?;
                let source = self.read_u8()?;

                match self.registers[source as usize] {
                    Value::Float(source_val) => match &mut self.registers[dest as usize] {
                        &mut Value::Float(ref mut dest_val) => {
                            *dest_val /= source_val;
                            Ok(None)
                        }
                        val => Err(VMError::WrongValueType(val.clone())),
                    },
                    ref val => Err(VMError::WrongValueType(val.clone())),
                }
            }
            0x80 => {
                // send_message
                let to = match self.registers[self.read_u8()? as usize] {
                    Value::ActorAddr(addr) => addr,
                    ref val => return Err(VMError::WrongValueType(val.clone())),
                };

                // TODO: Handle delay
                let _delay_reg = self.read_u8()?;

                let atom = match self.registers[self.read_u8()? as usize] {
                    Value::Atom(atom) => atom,
                    ref val => return Err(VMError::WrongValueType(val.clone())),
                };

                let n_args = self.read_u64()? as usize;
                let mut values = Vec::with_capacity(n_args);
                for _ in 0..n_args {
                    values.push(self.registers[self.read_u8()? as usize].clone());
                }

                Ok(Some(IO::SendMessage { to, atom, values }))
            }
            0x81 => {
                // add_handler
                let atom = match self.registers[self.read_u8()? as usize] {
                    Value::Atom(atom) => atom,
                    ref val => return Err(VMError::WrongValueType(val.clone())),
                };

                let program = self.read_u64()?;

                let n_presets = self.read_u64()? as usize;
                let mut presets = Vec::with_capacity(n_presets);
                for _ in 0..n_presets {
                    let reg_idx = self.read_u8()?;
                    let reg_value = self.registers[self.read_u8()? as usize].clone();
                    presets.push((reg_idx, reg_value));
                }

                Ok(Some(IO::AddHandler {
                    atom,
                    program,
                    presets,
                }))
            }
            0x82 => {
                // remove_handler
                let atom = match self.registers[self.read_u8()? as usize] {
                    Value::Atom(atom) => atom,
                    ref val => return Err(VMError::WrongValueType(val.clone())),
                };

                Ok(Some(IO::RemoveHandler { atom }))
            }
            unk_inst => Err(VMError::NoSuchInstruction(unk_inst)),
        }
    }
}

#[test]
fn test_send_message() {
    // Compiled from bytecode/examples/send.dig
    let code = include_bytes!("test-compiled-files/send.act") as &[u8];

    let io_state = IOState {
        self_addr: ActorAddr::random(),
    };
    let mut vm = VMState::new(code);

    for _ in 0..1000 {
        // limit after 1000 cycles
        match vm.step_once(&io_state) {
            Ok(None) => continue,
            Ok(Some(IO::Done)) => break,
            Ok(Some(IO::SendMessage { to, atom, values })) => {
                assert_eq!(to, io_state.self_addr);
                assert_eq!(atom, Atom(50));
                assert_eq!(values, vec![Value::Number(69), Value::Number(420)]);
            }
            Ok(Some(io)) => {
                assert!(false, "Unexpected IO: {:?}", io);
            }
            Err(err) => {
                assert!(false, "Unexpected error: {:?}", err);
            }
        }
    }
}

#[test]
fn test_self_modifier() {
    // Compiled from bytecode/examples/send.dig
    let code = include_bytes!("test-compiled-files/self-modifier.act") as &[u8];

    let io_state = IOState {
        self_addr: ActorAddr::random(),
    };
    let mut vm = VMState::new(code);

    for _ in 0..1000 {
        // limit after 1000 cycles
        match vm.step_once(&io_state) {
            Ok(None) => continue,
            Ok(Some(IO::Done)) => continue,
            Ok(Some(IO::AddHandler {
                atom,
                program,
                presets,
            })) => {
                assert_eq!(atom, Atom(400));
                assert_eq!(program, 43);
                assert_eq!(
                    presets,
                    vec![(1, Value::Number(20)), (2, Value::Number(30)),]
                );
            }
            Ok(Some(IO::RemoveHandler { atom })) => {
                assert_eq!(atom, Atom(400));
            }
            Ok(Some(io)) => {
                assert!(false, "Unexpected IO: {:?}", io);
            }
            Err(err) => {
                assert!(false, "Unexpected error: {:?}", err);
            }
        }
    }
}

#[test]
fn test_intops() {
    // set_integer r0 20
    // set_integer r1 30
    // add_int r0 r1
    // mul_int r0 r0
    // sub_int r1 r1
    #[rustfmt::skip]
    let code = &[
        0x02, 0x00, 0x14, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x02, 0x01, 0x1e, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x10, 0x00, 0x01,
        0x12, 0x00, 0x00,
        0x11, 0x01, 0x01,
    ];
    let io_state = IOState {
        self_addr: ActorAddr::random(),
    };
    let mut vm = VMState::new(code);

    // set_integer r0 20
    // set_integer r1 30
    assert_eq!(vm.step_once(&io_state), Ok(None));
    assert_eq!(vm.step_once(&io_state), Ok(None));

    assert_eq!(
        (&vm.registers[0], &vm.registers[1]),
        (&Value::Number(20), &Value::Number(30)),
    );

    // add_int r0 r1
    assert_eq!(vm.step_once(&io_state), Ok(None));
    assert_eq!(
        (&vm.registers[0], &vm.registers[1]),
        (&Value::Number(50), &Value::Number(30)),
    );

    // mul_int r0 r0
    assert_eq!(vm.step_once(&io_state), Ok(None));
    assert_eq!(
        (&vm.registers[0], &vm.registers[1]),
        (&Value::Number(2500), &Value::Number(30)),
    );

    // sub_int r1 r1
    assert_eq!(vm.step_once(&io_state), Ok(None));
    assert_eq!(
        (&vm.registers[0], &vm.registers[1]),
        (&Value::Number(2500), &Value::Number(0)),
    );
}

#[test]
// Comparing floats should be fine here, since we do the exact same operations
fn test_flops() {
    // set_float r0 20.5
    // set_float r1 29.5
    // add_float r0 r1
    // div_float r1 r0
    // mul_float r0 r0
    // sub_float r1 r1
    #[rustfmt::skip]
    let code = &[
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x34, 0x40, // 20.5 in hex
        0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x3d, 0x40, // 29.5 in hex
        0x20, 0x00, 0x01,
        0x23, 0x01, 0x00,
        0x22, 0x00, 0x00,
        0x21, 0x01, 0x01,
    ];
    let io_state = IOState {
        self_addr: ActorAddr::random(),
    };
    let mut vm = VMState::new(code);

    // set_float r0 20.5
    // set_float r1 29.5
    assert_eq!(vm.step_once(&io_state), Ok(None));
    assert_eq!(vm.step_once(&io_state), Ok(None));

    assert_eq!(
        (&vm.registers[0], &vm.registers[1]),
        (&Value::Float(20.5), &Value::Float(29.5)),
    );

    // add_float r0 r1
    assert_eq!(vm.step_once(&io_state), Ok(None));
    assert_eq!(
        (&vm.registers[0], &vm.registers[1]),
        (&Value::Float(50.0), &Value::Float(29.5)),
    );

    // div_float r1 r0
    assert_eq!(vm.step_once(&io_state), Ok(None));
    assert_eq!(
        (&vm.registers[0], &vm.registers[1]),
        (&Value::Float(50.0), &Value::Float(29.5 / 50.0)),
    );

    // mul_float r0 r0
    assert_eq!(vm.step_once(&io_state), Ok(None));
    assert_eq!(
        (&vm.registers[0], &vm.registers[1]),
        (&Value::Float(2500.0), &Value::Float(29.5 / 50.0)),
    );

    // sub_float r1 r1
    assert_eq!(vm.step_once(&io_state), Ok(None));
    assert_eq!(
        (&vm.registers[0], &vm.registers[1]),
        (&Value::Float(2500.0), &Value::Float(0.0)),
    );
}

#[test]
fn test_setters() {
    // set_self_addr r5
    // set_integer r3 20
    // copy r4 r3
    // integer_to_atom r3
    // integer_to_atom r5 ; type error!
    #[rustfmt::skip]
    let code = &[
        0x00, 0x05,
        0x02, 0x03, 0x14, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x04, 0x04, 0x03,
        0x06, 0x03,
        0x06, 0x05,
    ];
    let io_state = IOState {
        self_addr: ActorAddr::random(),
    };
    let mut vm = VMState::new(code);

    // set_self_addr r5
    assert_eq!(vm.step_once(&io_state), Ok(None));
    assert_eq!(vm.registers[5], Value::ActorAddr(io_state.self_addr));

    // set_integer r3 20
    assert_eq!(vm.step_once(&io_state), Ok(None));
    assert_eq!(vm.registers[3], Value::Number(20));

    // copy r4 r3
    assert_eq!(vm.step_once(&io_state), Ok(None));
    assert_eq!(vm.registers[4], vm.registers[3]);
    assert_eq!(vm.registers[4], Value::Number(20));

    // integer_to_atom r3
    assert_eq!(vm.step_once(&io_state), Ok(None));
    assert_eq!(vm.registers[3], Value::Atom(Atom(20)));

    // integer_to_atom r5
    assert_eq!(
        vm.step_once(&io_state),
        Err(VMError::WrongValueType(Value::ActorAddr(
            io_state.self_addr
        )))
    );
}
