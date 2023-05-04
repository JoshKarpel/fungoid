use std::{
    cmp::Ordering,
    convert::TryInto,
    error::Error,
    fmt::{Display, Formatter},
    io::{Read, Write},
};

use rand::{
    distributions::{Distribution, Standard},
    prelude::ThreadRng,
    thread_rng, Rng,
};
use time::{format_description, format_description::FormatItem, OffsetDateTime};

use crate::program::{Position, Program};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PointerDirection {
    Up,
    Down,
    Left,
    Right,
}

impl Distribution<PointerDirection> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> PointerDirection {
        match rng.gen_range(0..4) {
            0 => PointerDirection::Up,
            1 => PointerDirection::Down,
            2 => PointerDirection::Left,
            _ => PointerDirection::Right,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct InstructionPointer {
    pub position: Position,
    pub direction: PointerDirection,
}

impl InstructionPointer {
    fn new() -> Self {
        InstructionPointer {
            position: Position { x: 0, y: 0 },
            direction: PointerDirection::Right,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Stack(Vec<isize>);

impl Stack {
    fn new() -> Stack {
        Stack(Vec::<isize>::new())
    }

    fn push(&mut self, val: isize) {
        self.0.push(val);
    }

    fn pop(&mut self) -> isize {
        self.0.pop().unwrap_or(0)
    }

    fn join(&self, sep: &str) -> String {
        self.0
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(sep)
    }

    pub fn items(&self) -> Vec<isize> {
        self.0.clone()
    }
}

#[derive(Clone, Debug)]
pub enum ExecutionError {
    OutputFailed,
    InputFailed,
    UnrecognizedInstruction {
        position: Position,
        instruction: char,
    },
}

impl Display for ExecutionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionError::OutputFailed => {
                write!(f, "Failed to write output")
            }
            ExecutionError::InputFailed => {
                write!(f, "Failed to read input")
            }
            ExecutionError::UnrecognizedInstruction {
                position,
                instruction,
            } => {
                write!(
                    f,
                    "Unrecognized instruction at (x={}, y={}): '{}'",
                    position.x, position.y, instruction
                )
            }
        }
    }
}

impl Error for ExecutionError {}

pub type ExecutionResult = Result<(), ExecutionError>;

pub struct ExecutionState<R: Read, O: Write> {
    pub program: Program,
    pub pointer: InstructionPointer,
    pub stack: Stack,
    rng: ThreadRng,
    pub terminated: bool,
    string_mode: bool,
    trace: bool,
    pub instruction_count: u64,
    pub input: R,
    pub output: O,
}

lazy_static! {
    pub static ref TRACE_FORMAT: Vec<FormatItem<'static>> =
        format_description::parse_borrowed::<2>("[year]-[month]-[day] [hour]:[minute]:[second]")
            .unwrap();
}

impl<R: Read, O: Write> ExecutionState<R, O> {
    pub fn new(program: Program, trace: bool, input: R, output: O) -> Self {
        ExecutionState {
            program,
            pointer: InstructionPointer::new(),
            stack: Stack::new(),
            rng: thread_rng(),
            terminated: false,
            string_mode: false,
            trace,
            instruction_count: 0,
            input,
            output,
        }
    }

    pub fn reset(&mut self) {
        self.pointer = InstructionPointer::new();
        self.stack = Stack::new();
        self.rng = thread_rng();
        self.terminated = false;
        self.string_mode = false;
        self.instruction_count = 0;
    }

    pub fn run(&mut self) -> ExecutionResult {
        while !self.terminated {
            self.step()?;
        }

        Ok(())
    }

    fn trace(&self) {
        eprintln!(
            "{} [{:4}] ({:2}, {:2}) -> {} | {}",
            OffsetDateTime::now_local().unwrap(),
            self.instruction_count,
            self.pointer.position.x,
            self.pointer.position.y,
            self.program.get(&self.pointer.position),
            self.stack.join(" ")
        );
    }

    pub fn step(&mut self) -> ExecutionResult {
        if self.trace {
            self.trace();
        }

        self.instruction_count += 1;

        // execute instruction at pointer
        // https://esolangs.org/wiki/Befunge#Instructions
        match self.program.get(&self.pointer.position) {
            '"' => self.string_mode = !self.string_mode,
            c if self.string_mode => self.stack.push(isize::from(c as u8)),
            '^' => self.pointer.direction = PointerDirection::Up,
            'v' => self.pointer.direction = PointerDirection::Down,
            '>' => self.pointer.direction = PointerDirection::Right,
            '<' => self.pointer.direction = PointerDirection::Left,
            '?' => self.pointer.direction = self.rng.gen(),
            '_' => {
                // horizontal if
                let top = self.stack.pop();
                if top == 0 {
                    self.pointer.direction = PointerDirection::Right;
                } else {
                    self.pointer.direction = PointerDirection::Left;
                }
            }
            // vertical if
            '|' => {
                let top = self.stack.pop();
                if top == 0 {
                    self.pointer.direction = PointerDirection::Down;
                } else {
                    self.pointer.direction = PointerDirection::Up;
                }
            }
            // addition
            '+' => {
                let a = self.stack.pop();
                let b = self.stack.pop();
                self.stack.push(a + b);
            }
            // subtraction
            '-' => {
                let a = self.stack.pop();
                let b = self.stack.pop();
                self.stack.push(b - a);
            }
            // multiplication
            '*' => {
                let a = self.stack.pop();
                let b = self.stack.pop();
                self.stack.push(a * b);
            }
            // division
            '/' => {
                let a = self.stack.pop();
                let b = self.stack.pop();
                self.stack.push(b / a);
            }
            // modulo
            '%' => {
                let a = self.stack.pop();
                let b = self.stack.pop();
                self.stack.push(b % a);
            }
            // logical not
            '!' => {
                let b = self.stack.pop();
                if b == 0 {
                    self.stack.push(1);
                } else {
                    self.stack.push(0);
                }
            }
            // greater than
            '`' => {
                let a = self.stack.pop();
                let b = self.stack.pop();
                if let Ordering::Greater = b.cmp(&a) {
                    self.stack.push(1)
                } else {
                    self.stack.push(0);
                }
            }
            // duplicate top of self.stack
            ':' => {
                let a = self.stack.pop();
                self.stack.push(a);
                self.stack.push(a);
            }
            // swap top of self.stack
            '\\' => {
                let a = self.stack.pop();
                let b = self.stack.pop();
                self.stack.push(a);
                self.stack.push(b);
            }
            // discard top of self.stack
            '$' => {
                self.stack.pop();
            }
            '.' => {
                write!(self.output, "{}", self.stack.pop())
                    .map_err(|_| ExecutionError::OutputFailed)?;
            }
            ',' => {
                write!(self.output, "{}", self.stack.pop() as u8 as char)
                    .map_err(|_| ExecutionError::OutputFailed)?;
            }
            '#' => move_pointer(&mut self.pointer),
            // get
            'g' => {
                let y = self.stack.pop();
                let x = self.stack.pop();
                self.stack
                    .push(isize::from(self.program.get(&Position { x, y }) as u8));
            }
            // push
            'p' => {
                let y = self.stack.pop();
                let x = self.stack.pop();
                let v = self.stack.pop();
                self.program.set(&Position { x, y }, v as u8 as char);
            }
            // get int from user
            // TODO: does not actually work from stdin
            '&' => {
                let mut input = String::new();
                self.input
                    .read_to_string(&mut input)
                    .map_err(|_| ExecutionError::InputFailed)?;
                self.stack.push(input.trim().parse::<isize>().unwrap());
            }
            // get char from user
            // TODO: does not actually work from stdin
            '~' => {
                let mut input = String::new();
                self.input
                    .read_to_string(&mut input)
                    .map_err(|_| ExecutionError::InputFailed)?;
                self.stack
                    .push(isize::from(input.chars().next().unwrap() as u8));
            }
            '@' => {
                self.terminated = true;
                return Ok(()); // exit immediately (do not move the pointer when terminating)
            }
            c @ '0'..='9' => self.stack.push(c.to_digit(10).unwrap().try_into().unwrap()),
            ' ' => {}
            c => {
                return Err(ExecutionError::UnrecognizedInstruction {
                    position: self.pointer.position,
                    instruction: c,
                });
            }
        }

        move_pointer(&mut self.pointer);

        Ok(())
    }
}

fn move_pointer(pointer: &mut InstructionPointer) {
    match pointer.direction {
        PointerDirection::Up => pointer.position.y -= 1,
        PointerDirection::Down => pointer.position.y += 1,
        PointerDirection::Right => pointer.position.x += 1,
        PointerDirection::Left => pointer.position.x -= 1,
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{
        examples::{ERATOSTHENES, FACTORIAL, HELLO_WORLD, QUINE},
        execution::{ExecutionError, ExecutionState},
        program::{Position, Program},
    };

    pub type GenericResult = Result<(), Box<dyn std::error::Error>>;

    #[test]
    fn hello_world() -> GenericResult {
        let program = Program::from_str(HELLO_WORLD)?;
        let input = [];
        let output = Vec::new();
        let mut execution = ExecutionState::new(program, false, input.as_slice(), output);
        execution.run()?;
        assert_eq!(
            "Hello, World!\n",
            String::from_utf8(execution.output).unwrap()
        );

        Ok(())
    }

    #[test]
    fn sieve_of_eratosthenes() -> GenericResult {
        let program = Program::from_str(ERATOSTHENES)?;
        let input = [];
        let output = Vec::new();
        let mut execution = ExecutionState::new(program, false, input.as_slice(), output);
        execution.run()?;
        assert_eq!(
            "2357111317192329313741434753596167717379",
            String::from_utf8(execution.output).unwrap()
        );

        Ok(())
    }

    #[test]
    fn quine() -> GenericResult {
        let program = Program::from_str(QUINE)?;
        let input = [];
        let output = Vec::new();
        let mut execution = ExecutionState::new(program, false, input.as_slice(), output);
        execution.run()?;
        assert_eq!(
            QUINE.trim_end(),
            String::from_utf8(execution.output).unwrap()
        );

        Ok(())
    }

    #[test]
    fn factorial() -> GenericResult {
        let program = Program::from_str(FACTORIAL)?;
        let input = ["5".chars().next().unwrap() as u8];
        let output = Vec::new();
        let mut execution = ExecutionState::new(program, false, input.as_slice(), output);
        execution.run()?;
        assert_eq!("120", String::from_utf8(execution.output).unwrap());

        Ok(())
    }

    #[test]
    fn unknown_instruction() -> GenericResult {
        let program = Program::from_str("z")?;
        let input = [];
        let output = Vec::new();
        let mut execution = ExecutionState::new(program, false, input.as_slice(), output);
        let result = execution.run();
        assert!(
            if let Err(ExecutionError::UnrecognizedInstruction {
                position,
                instruction,
            }) = result
            {
                position == Position { x: 0, y: 0 } && instruction == 'z'
            } else {
                false
            }
        );

        Ok(())
    }
}
