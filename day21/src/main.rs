use std::collections::HashSet;
use std::error;
use std::fmt;
use std::fs;
use std::result;
use std::str::FromStr;

type Result<T> = std::result::Result<T, Box<error::Error>>;

#[derive(Debug)]
enum Error {
    InvalidInput,
    MemoryAccessViolation,
    Abort,
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

type Value = i64;
type Index = u32;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Registers {
    ip: Value,
    gpr: [Value; NUM_GPR],
}

impl Registers {
    fn new() -> Self {
        Registers {
            ip: 0,
            gpr: [0; NUM_GPR],
        }
    }

    fn get_unchecked(&self, idx: Index) -> Value {
        self.gpr[idx as usize]
    }

    fn get_ip(&self) -> Value {
        self.ip
    }

    fn set_unchecked(&mut self, idx: Index, val: Value) {
        self.gpr[idx as usize] = val;
    }

    fn set_ip(&mut self, val: Value) {
        self.ip = val;
    }
}

#[derive(Debug, Clone)]
struct Operands {
    a: Value,
    b: Value,
    c: Value,
}

#[derive(Debug, Clone)]
struct Instruction {
    opcode: String,
    ops: Operands,
}

impl Instruction {
    fn new(opcode: String, regs: &[Value]) -> Self {
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
        let nums: Vec<Value> = nums
            .split(' ')
            .map(|s| s.parse())
            .collect::<result::Result<_, _>>()?;

        Ok(Instruction::new(opcode, &nums))
    }
}

#[derive(Debug)]
struct Input {
    ipreg: Index,
    ins: Vec<Instruction>,
}

impl FromStr for Input {
    type Err = Box<error::Error>;

    fn from_str(s: &str) -> Result<Self> {
        let mut line_it = s.lines();
        let ipreg: Index = line_it
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

const OPS: &[&Fn(Value, Value) -> Value] = &[
    &|a, b| a + b,
    &|a, b| a * b,
    &|a, b| a & b,
    &|a, b| a | b,
    &|a, _| a,
    &|a, b| (a > b) as Value,
    &|a, b| (a == b) as Value,
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
    ipreg: Index,
    regs: Registers,

    part1: Option<Value>,
    prev_regs: Vec<Registers>,
    seen: HashSet<Registers>,
}

impl Iss {
    fn new(ipreg: Index) -> Self {
        Iss {
            ipreg,
            regs: Registers::new(),

            part1: None,
            prev_regs: vec![Registers::new(); 2],
            seen: HashSet::new(),
        }
    }

    fn insn(&mut self, opcode: Opcode, ops: &Operands) -> Result<()> {
        self.regs.set_unchecked(self.ipreg, self.regs.get_ip());

        let res = match opcode.2 {
            Mode::RegReg => OPS[opcode.1](
                self.regs.get_unchecked(ops.a as Index),
                self.regs.get_unchecked(ops.b as Index),
            ),
            Mode::RegImm => OPS[opcode.1](self.regs.get_unchecked(ops.a as Index), ops.b),
            Mode::ImmReg => OPS[opcode.1](ops.a, self.regs.get_unchecked(ops.b as Index)),
            Mode::ImmImm => OPS[opcode.1](ops.a, ops.b),
        };
        self.regs.set_unchecked(ops.c as Index, res);

        self.regs.set_ip(self.regs.get_unchecked(self.ipreg) + 1);

        Ok(())
    }

    fn run_cycle(&mut self, insn_mem: &[Instruction]) -> Result<()> {
        // Fetch instruction from memory
        let ins = insn_mem
            .get(self.regs.get_ip() as usize)
            .ok_or(Error::MemoryAccessViolation)?;

        for opcode in OPCODES {
            if ins.opcode == opcode.0 {
                if opcode.0 == "eqrr" {
                    if self.part1.is_none() {
                        let val = self.regs.get_unchecked(2);
                        println!("Part 1: {}", val);
                        self.part1 = Some(val);
                    }

                    if self.seen.contains(&self.regs) {
                        println!("Part 2: {}", self.prev_regs[0].get_unchecked(2));
                        return Err(Box::new(Error::Abort));
                    }
                    self.prev_regs.rotate_left(1);
                    self.prev_regs[1] = self.regs.clone();
                    self.seen.insert(self.regs.clone());
                }

                self.insn(*opcode, &ins.ops)?;
                break;
            }
        }

        Ok(())
    }

    #[allow(dead_code)]
    fn debug_insn(&mut self, opcode: Opcode, ops: &Operands) -> Result<()> {
        println!();
        println!("pc = {}", self.regs.get_ip());
        println!("opcode: {:?}", opcode);
        println!("ops: {:?}", ops);
        print!("Regs before: ");
        for i in 0..6 {
            print!("{:5} ", self.regs.get_unchecked(i));
        }
        println!();
        let res = self.insn(opcode, ops);
        print!("Regs after: ");
        for i in 0..6 {
            print!("{:5} ", self.regs.get_unchecked(i));
        }
        println!();
        res
    }
}

fn main() -> Result<()> {
    let input = fs::read_to_string("input")?;
    let input: Input = input.parse()?;

    let mut iss = Iss::new(input.ipreg);

    // iss.regs.set(0, 10780777)?;

    while iss.run_cycle(&input.ins).is_ok() {}

    Ok(())
}
