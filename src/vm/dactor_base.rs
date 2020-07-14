use std::convert::TryInto;

pub struct DactorBase {
    pub actor_code: Vec<u8>,
    pub programs: Vec<(usize, usize)>,
}

#[derive(Debug)]
pub enum ParseDactorError {
    UnexpectedEOF,
    ProgramOutOfRange(usize),
}

fn read_u64(inp: &mut &[u8]) -> Result<u64, ParseDactorError> {
    if inp.len() < 8 {
        return Err(ParseDactorError::UnexpectedEOF);
    }

    let bytes = &inp[0..8];
    *inp = &inp[8..];

    let value = u64::from_le_bytes(bytes.try_into().unwrap());

    Ok(value)
}

impl DactorBase {
    pub fn parse(mut inp: &[u8]) -> Result<DactorBase, ParseDactorError> {
        let orig_inp = inp;

        let n_programs = read_u64(&mut inp)?;
        let mut programs = Vec::with_capacity(n_programs as usize);

        for _ in 0..n_programs {
            let prog_start = read_u64(&mut inp)? as usize;
            let prog_end = read_u64(&mut inp)? as usize;

            if prog_start >= orig_inp.len() {
                return Err(ParseDactorError::ProgramOutOfRange(prog_start));
            }
            if prog_end > orig_inp.len() {
                return Err(ParseDactorError::ProgramOutOfRange(prog_end));
            }

            programs.push((prog_start, prog_end));
        }

        Ok(DactorBase { actor_code: orig_inp.to_vec(), programs })
    }
}
