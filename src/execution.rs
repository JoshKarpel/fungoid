use std::cmp::Ordering;
use std::convert::TryInto;
use std::io::{Read, Write};

use rand::distributions::{Distribution, Standard};
use rand::prelude::ThreadRng;
use rand::{thread_rng, Rng};

use crate::{Position, Program};

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

#[derive(Debug)]
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

pub struct ExecutionState<'input, 'output, O: Write> {
    pub program: Program,
    pub pointer: InstructionPointer,
    pub stack: Stack,
    rng: ThreadRng,
    pub terminated: bool,
    string_mode: bool,
    trace: bool,
    pub instruction_count: u64,
    input: &'input mut dyn Read,
    pub output: &'output mut O,
}

impl<'input, 'output, O: Write> ExecutionState<'input, 'output, O> {
    pub fn new(
        program: Program,
        trace: bool,
        input: &'input mut dyn Read,
        output: &'output mut O,
    ) -> Self {
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

    pub fn run(mut self) -> Self {
        while !self.terminated {
            self.step();
        }

        self
    }

    fn trace(&self) {
        eprintln!(
            "{} [{:4}] ({:2}, {:2}) -> {} | {}",
            chrono::Local::now().format("%F %T%.6f"),
            self.instruction_count,
            self.pointer.position.x,
            self.pointer.position.y,
            self.program.get(&self.pointer.position),
            self.stack.join(" ")
        );
    }

    pub fn step(&mut self) {
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
                write!(self.output, "{}", self.stack.pop()).expect("Failed to write int");
            }
            ',' => {
                write!(self.output, "{}", self.stack.pop() as u8 as char)
                    .expect("Failed to write char");
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
            '&' => {
                let mut input = String::new();
                self.input
                    .read_to_string(&mut input)
                    .expect("failed to read int");
                self.stack.push(input.trim().parse::<isize>().unwrap());
            }
            // get char from user
            '~' => {
                let mut input = String::new();
                self.input
                    .read_to_string(&mut input)
                    .expect("failed to read char");
                self.stack
                    .push(isize::from(input.chars().next().unwrap() as u8));
            }
            '@' => {
                self.terminated = true;
                return; // exit immediately (do not move the pointer when terminating)
            }
            c @ '0'..='9' => self.stack.push(c.to_digit(10).unwrap().try_into().unwrap()),
            ' ' => {}
            c => panic!("Unrecognized instruction! {}", c),
        }

        move_pointer(&mut self.pointer);
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
