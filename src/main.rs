extern crate rand;
#[macro_use]
extern crate rand_derive;
extern crate time;
extern crate separator;

use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io;
use std::fmt;
use std::cmp::Ordering;

use rand::Rng;
use time::PreciseTime;
use separator::Separatable;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];

    let program = Program::from_file(&filename);

    println!("{}", vec!["-"; 80].join(""));
    println!("{}", program);
    println!("{}", vec!["-"; 80].join(""));

    println!("OUTPUT");
//    run(program);
    benchmark(program);
}

struct Program([[char; 80]; 30]);


impl Program {
    fn new() -> Program {
        Program([[' '; 80]; 30])
    }

    fn from_str(s: &str) -> Program {
        let mut program = Program::new();

        for (y, line) in s.split('\n').enumerate() {
            for (x, ch) in line.chars().enumerate() {
                program.set(&Position { x, y }, ch);
            }
        }

        program
    }

    fn from_file(path: &str) -> Program {
        let mut f = File::open(path).expect("file not found");
        let mut contents = String::new();
        f.read_to_string(&mut contents).expect("failed to read file");

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

#[derive(Debug, Copy, Clone, Rand, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
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

fn run(mut program: Program) -> u64 {
    let mut pointer = InstructionPointer::new();
    let mut stack = Stack::new();

    let mut rng = rand::thread_rng();

    let mut instruction_count: u64 = 0;

    let mut string_mode = false;

    loop {
        instruction_count += 1;

//        println!("{:?} : {:?}", program.get(&pointer.position), stack);

        // execute instruction at pointer
        // https://esolangs.org/wiki/Befunge#Instructions
        match program.get(&pointer.position) {
            '"' => string_mode = !string_mode,
            c if string_mode => stack.push(i64::from(c as u8)),
            '^' => pointer.direction = Direction::Up,
            'v' => pointer.direction = Direction::Down,
            '>' => pointer.direction = Direction::Right,
            '<' => pointer.direction = Direction::Left,
            '?' => pointer.direction = rng.gen(),
            '_' => { // horizontal if
                let top = stack.pop();
                if top == 0 {
                    pointer.direction = Direction::Right;
                } else {
                    pointer.direction = Direction::Left;
                }
            }
            '|' => { // vertical if
                let top = stack.pop();
                if top == 0 {
                    pointer.direction = Direction::Down;
                } else {
                    pointer.direction = Direction::Up;
                }
            }
            '+' => { // addition
                let a = stack.pop();
                let b = stack.pop();
                stack.push(a + b);
            }
            '-' => { // subtraction
                let a = stack.pop();
                let b = stack.pop();
                stack.push(b - a);
            }
            '*' => { // multiplication
                let a = stack.pop();
                let b = stack.pop();
                stack.push(a * b);
            }
            '/' => { // division
                let a = stack.pop();
                let b = stack.pop();
                stack.push(b / a);
            }
            '%' => { // modulo
                let a = stack.pop();
                let b = stack.pop();
                stack.push(b % a);
            }
            '!' => { // logical not
                let b = stack.pop();
                if b == 0 {
                    stack.push(1);
                } else {
                    stack.push(0);
                }
            }
            '`' => { // greater than
                let a = stack.pop();
                let b = stack.pop();
                if let Ordering::Greater = b.cmp(&a) {
                    stack.push(1)
                } else {
                    stack.push(0);
                }
            }
            ':' => { // duplicate top of stack
                let a = stack.pop();
                stack.push(a);
                stack.push(a);
            }
            '\\' => { // swap top of stack
                let a = stack.pop();
                let b = stack.pop();
                stack.push(a);
                stack.push(b);
            }
            '$' => { // discard top of stack
                stack.pop();
            }
            '.' => print!("{}", stack.pop()),
            ',' => print!("{}", stack.pop() as u8 as char),
            '#' => move_pointer(&mut pointer),
            'g' => { // get
                let y = stack.pop();
                let x = stack.pop();
                stack.push(i64::from(program.get(&Position { x: x as usize, y: y as usize }) as u8));
            }
            'p' => { // push
                let y = stack.pop();
                let x = stack.pop();
                let v = stack.pop();
                program.set(&Position { x: x as usize, y: y as usize }, v as u8 as char);
            }
            '&' => { // get int from user
                let mut input = String::new();
                io::stdin().read_line(&mut input).expect("failed to read int");
                println!("{}", input);
                stack.push(input.trim().parse::<i64>().unwrap());
            }
            '~' => { // get char from user
                let mut input = String::new();
                io::stdin().read_line(&mut input).expect("failed to read char");
                stack.push(i64::from(input.chars().next().unwrap() as u8));
            }
            '@' => break,
            c @ '0'...'9' => stack.push(i64::from(c.to_digit(10).unwrap())),
            ' ' => {}
            c => { panic!("Unrecognized instruction! {}", c) }
        }

        move_pointer(&mut pointer);
    }

    instruction_count
}

fn move_pointer(pointer: &mut InstructionPointer) {
    match pointer.direction {
        Direction::Up => { pointer.position.y -= 1 }
        Direction::Down => { pointer.position.y += 1 }
        Direction::Right => { pointer.position.x += 1 }
        Direction::Left => { pointer.position.x -= 1 }
    }
}

fn benchmark(program: Program) {
    let start = PreciseTime::now();
    let instruction_count = run(program);
    let end = PreciseTime::now();

    let duration = start.to(end);
    println!("\n");
    println!("Executed {:?} instructions in {:?} μs", instruction_count, duration.num_microseconds().unwrap());
    let num_seconds = 1.0e-9 * duration.num_nanoseconds().unwrap() as f64;
    println!("Running at {} instructions/second", ((instruction_count as f64 / num_seconds) as u64).separated_string());
}
