use std::error;
use std::fmt;
use std::fs;
use std::result;
use std::str::FromStr;

type Result<T> = std::result::Result<T, Box<error::Error>>;

#[derive(Debug)]
enum Error {
    InvalidInput,
    IllegalInstruction,
    MemoryAccessViolation,
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

const NUM_GPR: usize = 6;

#[derive(Debug, Clone, PartialEq)]
struct Registers {
    ip: u32,
    gpr: [u32; NUM_GPR],
}

impl Registers {
    fn new() -> Self {
        Registers {
            ip: 0,
            gpr: [0; NUM_GPR],
        }
    }

    fn get(&self, idx: u32) -> Result<&u32> {
        if idx as usize >= NUM_GPR {
            return Err(Box::new(Error::IllegalInstruction));
        }

        Ok(&self.gpr[idx as usize])
    }

    fn get_unchecked(&self, idx: u32) -> u32 {
        self.gpr[idx as usize]
    }

    fn get_ip(&self) -> u32 {
        self.ip
    }

    fn set(&mut self, idx: u32, val: u32) -> Result<()> {
        if idx as usize >= NUM_GPR {
            return Err(Box::new(Error::IllegalInstruction));
        }

        self.gpr[idx as usize] = val;
        Ok(())
    }

    fn set_unchecked(&mut self, idx: u32, val: u32) {
        self.gpr[idx as usize] = val;
    }

    fn set_ip(&mut self, val: u32) {
        self.ip = val;
    }
}

#[derive(Debug, Clone)]
struct Operands {
    a: u32,
    b: u32,
    c: u32,
}

#[derive(Debug, Clone)]
struct Instruction {
    opcode: String,
    ops: Operands,
}

impl Instruction {
    fn new(opcode: String, regs: &[u32]) -> Self {
        assert!(regs.len() == 3);
        Instruction {
            opcode,
            ops: Operands {
                a: regs[0],
                b: regs[1],
                c: regs[2],
            },
        }
    }
}

impl FromStr for Instruction {
    type Err = Box<error::Error>;

    fn from_str(s: &str) -> Result<Self> {
        let mut opcode = String::new();
        let nums: String = s
            .chars()
            .skip_while(|c| {
                if *c != ' ' {
                    opcode.push(*c);
                    true
                } else {
                    false
                }
            })
            .skip(1)
            .collect();
        let nums: Vec<u32> = nums
            .split(' ')
            .map(|s| s.parse())
            .collect::<result::Result<_, _>>()?;

        Ok(Instruction::new(opcode, &nums))
    }
}

#[derive(Debug)]
struct Input {
    ipreg: u32,
    ins: Vec<Instruction>,
}

impl FromStr for Input {
    type Err = Box<error::Error>;

    fn from_str(s: &str) -> Result<Self> {
        let mut line_it = s.lines();
        let ipreg: u32 = line_it
            .next()
            .ok_or(Error::InvalidInput)?
            .replace("#ip ", "")
            .parse()?;

        let ins: Vec<Instruction> = line_it.map(|l| l.parse()).collect::<Result<_>>()?;

        Ok(Input { ipreg, ins })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Mode {
    RegReg,
    RegImm,
    ImmReg,
    ImmImm,
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

type Opcode = (&'static str, usize, Mode);

const OPCODES: &[Opcode] = &[
    ("addr", 0, Mode::RegReg),
    ("addi", 0, Mode::RegImm),
    ("mulr", 1, Mode::RegReg),
    ("muli", 1, Mode::RegImm),
    ("banr", 2, Mode::RegReg),
    ("bani", 2, Mode::RegImm),
    ("borr", 3, Mode::RegReg),
    ("bori", 3, Mode::RegImm),
    ("setr", 4, Mode::RegImm),
    ("seti", 4, Mode::ImmImm),
    ("gtir", 5, Mode::ImmReg),
    ("gtri", 5, Mode::RegImm),
    ("gtrr", 5, Mode::RegReg),
    ("eqir", 6, Mode::ImmReg),
    ("eqri", 6, Mode::RegImm),
    ("eqrr", 6, Mode::RegReg),
];

#[derive(Debug, Clone)]
struct Iss {
    ipreg: u32,
    regs: Registers,
    insn_mem: Vec<Instruction>,
    do_haxx: bool,
}

impl Iss {
    fn new(ipreg: u32, insn_mem: Vec<Instruction>) -> Self {
        Iss {
            ipreg,
            regs: Registers::new(),
            insn_mem,
            do_haxx: false,
        }
    }

    fn enable_haxx(&mut self) {
        self.do_haxx = true;
    }

    fn insn(&mut self, opcode: Opcode, ops: &Operands) -> Result<()> {
        self.regs.set_unchecked(self.ipreg, self.regs.get_ip());

        if self.do_haxx {
            if self.regs.ip == 4 && self.regs.gpr[4] > self.regs.gpr[5] {
                self.regs.ip = 12;
                return Ok(());
            }
        }

        let res = match opcode.2 {
            Mode::RegReg => OPS[opcode.1](
                self.regs.get_unchecked(ops.a),
                self.regs.get_unchecked(ops.b),
            ),
            Mode::RegImm => OPS[opcode.1](self.regs.get_unchecked(ops.a), ops.b),
            Mode::ImmReg => OPS[opcode.1](ops.a, self.regs.get_unchecked(ops.b)),
            Mode::ImmImm => OPS[opcode.1](ops.a, ops.b),
        };
        self.regs.set_unchecked(ops.c, res);

        self.regs.set_ip(self.regs.get_unchecked(self.ipreg) + 1);

        Ok(())
    }

    fn run_cycle(&mut self) -> Result<()> {
        // Fetch instruction from memory
        let ins = self
            .insn_mem
            .get(self.regs.get_ip() as usize)
            .ok_or(Error::MemoryAccessViolation)?
            .clone();

        for opcode in OPCODES {
            if ins.opcode == opcode.0 {
                self.insn(*opcode, &ins.ops)?;
                break;
            }
        }

        Ok(())
    }
}

fn main() -> Result<()> {
    let input = fs::read_to_string("input")?;
    let input: Input = input.parse()?;

    let mut iss = Iss::new(input.ipreg, input.ins);

    let mut tmp = iss.clone();
    while tmp.run_cycle().is_ok() {}
    println!("Part 1: {}", tmp.regs.get(0)?);

    iss.regs.set(0, 1)?;
    iss.enable_haxx();
    while iss.run_cycle().is_ok() {}
    println!("Part 2: {}", iss.regs.get(0)?);

    Ok(())
}
