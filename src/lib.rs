extern crate rand;
extern crate separator;
extern crate time;

use std::{cmp::Ordering, fmt, fs::File, io, io::prelude::*};

use rand::{
    distributions::{Distribution, Standard},
    prelude::*,
    Rng,
};
use separator::Separatable;
use time::PreciseTime;

#[derive(Copy, Clone)]
pub struct Program([[char; 80]; 30]);

impl Program {
    fn new() -> Program {
        Program([[' '; 80]; 30])
    }

    pub fn from_str(s: &str) -> Program {
        let mut program = Program::new();

        for (y, line) in s.split('\n').enumerate() {
            for (x, ch) in line.chars().enumerate() {
                program.set(&Position { x, y }, ch);
            }
        }

        program
    }

    pub fn from_file(path: &str) -> Program {
        let mut f = File::open(path).expect("file not found");
        let mut contents = String::new();
        f.read_to_string(&mut contents)
            .expect("failed to read file");

        Program::from_str(&contents)
    }

    fn get(&self, pos: &Position) -> char {
        self.0[pos.y][pos.x]
    }

    fn set(&mut self, pos: &Position, c: char) {
        self.0[pos.y][pos.x] = c;
    }
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut chars = Vec::new();

        for line in &self.0 {
            for c in line.iter() {
                chars.push(c.to_string());
            }
            chars.push('\n'.to_string());
        }

        write!(f, "{}", chars.join(""))
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct Position {
    x: usize,
    y: usize,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Distribution<Direction> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Direction {
        match rng.gen_range(0, 4) {
            0 => Direction::Up,
            1 => Direction::Down,
            2 => Direction::Left,
            _ => Direction::Right,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct InstructionPointer {
    position: Position,
    direction: Direction,
}

impl InstructionPointer {
    fn new() -> InstructionPointer {
        InstructionPointer {
            position: Position { x: 0, y: 0 },
            direction: Direction::Right,
        }
    }
}

#[derive(Debug)]
struct Stack(Vec<i64>);

impl Stack {
    fn new() -> Stack {
        Stack(Vec::<i64>::new())
    }

    fn push(&mut self, val: i64) {
        self.0.push(val);
    }

    fn pop(&mut self) -> i64 {
        self.0.pop().unwrap_or(0)
    }
}

pub struct ProgramState<'input, 'output, 'error> {
    program: Program,
    pointer: InstructionPointer,
    stack: Stack,
    rng: ThreadRng,
    string_mode: bool,
    input: &'input mut dyn Read,
    output: &'output mut dyn Write,
    error: &'error mut dyn Write,
}

impl<'input, 'output, 'error> ProgramState<'input, 'output, 'error> {
    fn new(
        program: Program,
        input: &'input mut dyn Read,
        output: &'output mut dyn Write,
        error: &'error mut dyn Write,
    ) -> Self {
        ProgramState {
            program,
            pointer: InstructionPointer::new(),
            stack: Stack::new(),
            rng: rand::thread_rng(),
            string_mode: false,
            input,
            output,
            error,
        }
    }

    fn run(&mut self) -> i64 {
        let mut instruction_count = 0;
        loop {
            instruction_count += 1;

            // println!("{:?} : {:?}", program.get(&pointer.position), self.stack);

            // execute instruction at pointer
            // https://esolangs.org/wiki/Befunge#Instructions
            match self.program.get(&self.pointer.position) {
                '"' => self.string_mode = !self.string_mode,
                c if self.string_mode => self.stack.push(i64::from(c as u8)),
                '^' => self.pointer.direction = Direction::Up,
                'v' => self.pointer.direction = Direction::Down,
                '>' => self.pointer.direction = Direction::Right,
                '<' => self.pointer.direction = Direction::Left,
                '?' => self.pointer.direction = self.rng.gen(),
                '_' => {
                    // horizontal if
                    let top = self.stack.pop();
                    if top == 0 {
                        self.pointer.direction = Direction::Right;
                    } else {
                        self.pointer.direction = Direction::Left;
                    }
                }
                // vertical if
                '|' => {
                    let top = self.stack.pop();
                    if top == 0 {
                        self.pointer.direction = Direction::Down;
                    } else {
                        self.pointer.direction = Direction::Up;
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
                    ()
                }
                ',' => {
                    write!(self.output, "{}", self.stack.pop() as u8 as char)
                        .expect("Failed to write char");
                    ()
                }
                '#' => move_pointer(&mut self.pointer),
                // get
                'g' => {
                    let y = self.stack.pop();
                    let x = self.stack.pop();
                    self.stack.push(i64::from(self.program.get(&Position {
                        x: x as usize,
                        y: y as usize,
                    }) as u8));
                }
                // push
                'p' => {
                    let y = self.stack.pop();
                    let x = self.stack.pop();
                    let v = self.stack.pop();
                    self.program.set(
                        &Position {
                            x: x as usize,
                            y: y as usize,
                        },
                        v as u8 as char,
                    );
                }
                // get int from user
                '&' => {
                    let mut input = String::new();
                    self.input
                        .read_to_string(&mut input)
                        .expect("failed to read int");
                    self.stack.push(input.trim().parse::<i64>().unwrap());
                }
                // get char from user
                '~' => {
                    let mut input = String::new();
                    self.input
                        .read_to_string(&mut input)
                        .expect("failed to read char");
                    self.stack
                        .push(i64::from(input.chars().next().unwrap() as u8));
                }
                '@' => break,
                c @ '0'..='9' => self.stack.push(i64::from(c.to_digit(10).unwrap())),
                ' ' => {}
                c => panic!("Unrecognized instruction! {}", c),
            }

            move_pointer(&mut self.pointer);
        }

        instruction_count
    }
}

fn move_pointer(pointer: &mut InstructionPointer) {
    match pointer.direction {
        Direction::Up => pointer.position.y -= 1,
        Direction::Down => pointer.position.y += 1,
        Direction::Right => pointer.position.x += 1,
        Direction::Left => pointer.position.x -= 1,
    }
}

pub fn run(program: Program) -> i64 {
    ProgramState::new(
        program,
        &mut io::stdin(),
        &mut io::stdout(),
        &mut io::stderr(),
    )
    .run()
}

pub fn time(program: Program) {
    let start = PreciseTime::now();
    let instruction_count = run(program);
    let end = PreciseTime::now();

    let duration = start.to(end);
    println!("");
    println!(
        "Executed {:?} instructions in {:?} μs",
        instruction_count,
        duration.num_microseconds().unwrap()
    );
    let num_seconds = 1.0e-9 * duration.num_nanoseconds().unwrap() as f64;
    println!(
        "Running at {} instructions/second",
        ((instruction_count as f64 / num_seconds) as u64).separated_string()
    );
}

#[cfg(test)]
mod tests {
    use crate::{Program, ProgramState};

    use std::io;

    #[test]
    fn hello_world() {
        let program = Program::from_str(
            &vec![
                r#">25*"!dlrow ,olleH":v "#,
                r#"                 v:,_@"#,
                r#"                 >  ^ "#,
            ]
            .join("\n"),
        );
        let mut output = Vec::new();
        ProgramState::new(program, &mut io::stdin(), &mut output, &mut Vec::new()).run();
        println!("{:?}", output);
        assert_eq!("Hello, world!\n", String::from_utf8(output).unwrap());
    }
}
