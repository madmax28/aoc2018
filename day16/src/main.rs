use std::collections::HashMap;
use std::error;
use std::fmt;
use std::fs;
use std::result;
use std::str::FromStr;

type Result<T> = std::result::Result<T, Box<error::Error>>;

#[derive(Debug)]
enum Error {
    IllegalInstruction,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl error::Error for Error {
    fn cause(&self) -> Option<&error::Error> {
        Some(self)
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Registers {
    a: u32,
    b: u32,
    c: u32,
    d: u32,
}

impl Registers {
    fn new() -> Self {
        Registers {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
        }
    }

    fn with_values(a: u32, b: u32, c: u32, d: u32) -> Self {
        Registers { a, b, c, d }
    }

    fn from_slice(v: &[u32]) -> Self {
        if v.len() != 4 {
            panic!("invalid sized vec");
        }

        Registers::with_values(v[0], v[1], v[2], v[3])
    }

    fn get(&self, idx: u32) -> Result<&u32> {
        match idx {
            0 => Ok(&self.a),
            1 => Ok(&self.b),
            2 => Ok(&self.c),
            3 => Ok(&self.d),
            _ => Err(Box::new(Error::IllegalInstruction)),
        }
    }

    fn set(&mut self, idx: u32, val: u32) -> Result<()> {
        match idx {
            0 => self.a = val,
            1 => self.b = val,
            2 => self.c = val,
            3 => self.d = val,
            _ => return Err(Box::new(Error::IllegalInstruction)),
        };

        Ok(())
    }
}

impl FromStr for Registers {
    type Err = Box<error::Error>;

    fn from_str(s: &str) -> Result<Self> {
        let token: String = s
            .chars()
            .skip_while(|c| *c != '[')
            .skip(1)
            .take_while(|c| *c != ']')
            .collect();

        let nums: Vec<u32> = token
            .split(',')
            .map(|s| s.trim().parse())
            .collect::<result::Result<_, _>>()?;

        Ok(Registers::from_slice(&nums))
    }
}

#[derive(Debug)]
struct Operands {
    a: u32,
    b: u32,
    c: u32,
}

#[derive(Debug)]
struct Instruction {
    opcode: u32,
    ops: Operands,
}

impl Instruction {
    fn new(opcode: u32, a: u32, b: u32, c: u32) -> Self {
        Instruction {
            opcode,
            ops: Operands { a, b, c },
        }
    }

    fn from_slice(v: &[u32]) -> Self {
        if v.len() != 4 {
            panic!("invalid vec size");
        }

        Instruction::new(v[0], v[1], v[2], v[3])
    }
}

impl FromStr for Instruction {
    type Err = Box<error::Error>;

    fn from_str(s: &str) -> Result<Self> {
        let nums: Vec<u32> = s
            .split(' ')
            .map(|s| s.parse())
            .collect::<result::Result<_, _>>()?;

        Ok(Instruction::from_slice(&nums))
    }
}

#[derive(Debug)]
struct Observation {
    regs_before: Registers,
    insn: Instruction,
    regs_after: Registers,
}

#[derive(Debug)]
struct Input {
    obs: Vec<Observation>,
    ins: Vec<Instruction>,
}

impl FromStr for Input {
    type Err = Box<error::Error>;

    fn from_str(s: &str) -> Result<Self> {
        let mut obs: Vec<Observation> = Vec::new();
        let mut ins: Vec<Instruction> = Vec::new();

        let mut line_it = s.lines();
        while let Some(mut l) = line_it.next() {
            if l.contains("Before") {
                let regs_before: Registers = l.parse()?;
                l = line_it.next().expect("Invalid input");
                let insn: Instruction = l.parse()?;
                l = line_it.next().expect("Invalid input");
                let regs_after: Registers = l.parse()?;

                obs.push(Observation {
                    regs_before,
                    insn,
                    regs_after,
                });
            } else if !l.is_empty() {
                ins.push(l.parse()?);
            }
        }

        Ok(Input { obs, ins })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Mode {
    RegReg,
    RegImm,
    ImmReg,
}

const OPS: &[&Fn(u32, u32) -> u32] = &[
    &|a, b| a + b,
    &|a, b| a * b,
    &|a, b| a & b,
    &|a, b| a | b,
    &|a, _| a,
    &|a, b| (a > b) as u32,
    &|a, b| (a == b) as u32,
];

type Opcode = (usize, Mode);

const OPCODES: &[Opcode] = &[
    (0, Mode::RegReg), // addr
    (0, Mode::RegImm), // addi
    (1, Mode::RegReg), // mulr
    (1, Mode::RegImm), // muli
    (2, Mode::RegReg), // banr
    (2, Mode::RegImm), // bani
    (3, Mode::RegReg), // borr
    (3, Mode::RegImm), // bori
    (4, Mode::RegReg), // setr
    (4, Mode::ImmReg), // seti
    (5, Mode::ImmReg), // gtir
    (5, Mode::RegImm), // gtri
    (5, Mode::RegReg), // gtrr
    (6, Mode::ImmReg), // eqir
    (6, Mode::RegImm), // eqri
    (6, Mode::RegReg), // eqrr
];

struct Iss {
    regs: Registers,
    opcodes: HashMap<u32, Opcode>,
}

impl Iss {
    fn new() -> Self {
        Iss {
            regs: Registers::new(),
            opcodes: HashMap::new(),
        }
    }

    fn insn(&mut self, opcode: Opcode, ops: &Operands) -> Result<()> {
        let res = match opcode.1 {
            Mode::RegReg => OPS[opcode.0](*self.regs.get(ops.a)?, *self.regs.get(ops.b)?),
            Mode::RegImm => OPS[opcode.0](*self.regs.get(ops.a)?, ops.b),
            Mode::ImmReg => OPS[opcode.0](ops.a, *self.regs.get(ops.b)?),
        };
        self.regs.set(ops.c, res)?;
        Ok(())
    }

    fn get_possible_opcodes(obs: &Observation) -> Vec<Opcode> {
        let mut iss = Iss::new();
        let mut opcodes: Vec<Opcode> = Vec::new();

        for opcode in OPCODES {
            iss.regs = obs.regs_before.clone();
            if iss.insn(*opcode, &obs.insn.ops).is_ok() && iss.regs == obs.regs_after {
                opcodes.push(*opcode);
            }
        }

        opcodes
    }

    fn find_opcodes(&mut self, obs: &[Observation]) -> Result<()> {
        let mut state: HashMap<u32, Vec<Opcode>> = HashMap::new();
        for i in 0..OPCODES.len() {
            state.insert(i as u32, OPCODES.to_vec());
        }

        for o in obs {
            let possible = Iss::get_possible_opcodes(o);
            let current = state.get_mut(&o.insn.opcode).expect("invalid opcode");

            for idx in (0..current.len()).rev() {
                if !possible.contains(&current[idx]) {
                    current.remove(idx);
                }
            }
        }

        while !state.is_empty() {
            state = state
                .into_iter()
                .filter_map(|(k, vs)| {
                    assert!(!vs.is_empty());
                    if vs.len() == 1 {
                        self.opcodes.insert(k, vs[0]);
                    }

                    let vs: Vec<_> = vs
                        .into_iter()
                        .filter(|v| !self.opcodes.values().any(|v2| v2 == v))
                        .collect();

                    if !vs.is_empty() {
                        Some((k, vs))
                    } else {
                        None
                    }
                })
                .collect();
        }

        Ok(())
    }

    fn execute_program(&mut self, instructions: &[Instruction]) -> Result<()> {
        for i in instructions {
            self.insn(
                *self.opcodes.get(&i.opcode).expect("unknown opcode"),
                &i.ops,
            )?;
        }

        Ok(())
    }

    #[allow(dead_code)]
    fn debug_insn(opcode: Opcode, obs: &Observation) {
        println!("opcode: {:?}", opcode);
        println!("ops: {:?}", obs.insn.ops);
        let mut iss = Iss::new();
        iss.regs = obs.regs_before.clone();
        println!(
            "Regs before: {:3} {:3} {:3} {:3}",
            iss.regs.a, iss.regs.b, iss.regs.c, iss.regs.d
        );
        iss.insn(opcode, &obs.insn.ops)
            .expect("illegal instruction");
        println!(
            "Regs after:  {:3} {:3} {:3} {:3}",
            iss.regs.a, iss.regs.b, iss.regs.c, iss.regs.d
        );
    }
}

fn main() -> Result<()> {
    let input = fs::read_to_string("input")?;
    let input: Input = input.parse()?;

    let mut cnt = 0;
    for o in &input.obs {
        if Iss::get_possible_opcodes(o).len() > 2 {
            cnt += 1;
        }
    }
    println!("Part 1: {}", cnt);

    let mut iss = Iss::new();
    iss.find_opcodes(&input.obs)?;
    iss.execute_program(&input.ins)?;
    println!("Part 1: {}", iss.regs.a);

    Ok(())
}
