use std::convert::TryInto;

use crate::messages::{ActorAddr, Atom, Value};

#[derive(Debug, Eq, PartialEq)]
pub enum IO {
    SendMessage {
        to: ActorAddr,
        // TODO
        // delay: Duration
        atom: Atom,
        values: Vec<Value>,
    },
    AddHandler {
        atom: Atom,
        program: u64,
        presets: Vec<(u8, Value)>,
    },
    RemoveHandler {
        atom: Atom,
    },
    Exit,
}

#[derive(Debug, Eq, PartialEq)]
pub enum ValueType {
    String,
    Integer,
}

#[derive(Debug, Eq, PartialEq)]
pub enum VMError {
    OutOfBounds,
    NoSuchInstruction(u8),
    WrongValueType(Value),
}

#[derive(Debug, Eq, PartialEq)]
pub struct IOState {
    self_addr: ActorAddr,
    // TODO
    // rng
}

pub struct VMState<'a> {
    instructions: &'a [u8],
    instruction_pointer: usize,

    registers: [Value; 256],
}

impl<'a> VMState<'a> {
    pub fn new(instructions: &'a [u8]) -> VMState {
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
            instructions,
            instruction_pointer: 0,
            registers,
        }
    }

    pub fn read_u8(&mut self) -> Result<u8, VMError> {
        if self.instruction_pointer >= self.instructions.len() {
            return Err(VMError::OutOfBounds);
        }
        let value = self.instructions[self.instruction_pointer];
        self.instruction_pointer += 1;

        Ok(value)
    }

    fn read_u64(&mut self) -> Result<u64, VMError> {
        if self.instruction_pointer + 8 > self.instructions.len() {
            return Err(VMError::OutOfBounds);
        }
        let bytes = &self.instructions[self.instruction_pointer..self.instruction_pointer + 8];
        self.instruction_pointer += 8;

        let value = u64::from_le_bytes(bytes.try_into().unwrap());

        Ok(value)
    }

    pub fn step_once(&mut self, io_state: &IOState) -> Result<Option<IO>, VMError> {
        match self.read_u8()? {
            0x00 => {
                // set_self_addr
                let reg = self.read_u8()?;
                self.registers[reg as usize] = Value::ActorAddr(io_state.self_addr);

                Ok(None)
            }
            // TODO: 0x01/set_float
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
            unk_inst => Err(VMError::NoSuchInstruction(unk_inst)),
        }
    }
}

