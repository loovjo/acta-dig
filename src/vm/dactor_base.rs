use std::convert::TryInto;

pub struct DactorBase {
    pub programs: Vec<Vec<u8>>,
}

pub enum ParseDactorError {
    UnexpectedEOF,
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
        let n_programs = read_u64(&mut inp)?;
        let mut programs = Vec::with_capacity(n_programs as usize);

        for _ in 0..n_programs {
            let prog_start = read_u64(&mut inp)?;
            let prog_end = read_u64(&mut inp)?;

            let program = &inp[prog_start as usize..prog_end as usize];
            programs.push(program.to_vec());
        }

        Ok(DactorBase { programs })
    }
}
