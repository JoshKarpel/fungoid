extern crate rand;
#[macro_use]
extern crate rand_derive;
extern crate time;


use std::fmt;
use std::cmp::Ordering;

use rand::Rng;
use time::PreciseTime;

fn main() {
//    let program = Program::from_str("0956+++.@");
    let program = Program::from_str(&vec![r#"5>:1-:v v *_$.@ "#, r#" ^    _$>\:^"#].join("\n"));
//    let program = Program::from_str(r#"64+"!dlroW ,olleH">:#,_@"#);

    println!("PROGRAM");
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

        for (y, line) in s.split("\n").enumerate() {
            for (x, ch) in line.chars().enumerate() {
                program.insert(&Position { x, y }, ch);
            }
        }

        program
    }

    fn insert(&mut self, pos: &Position, c: char) {
        self.0[pos.y][pos.x] = c;
    }

    fn get(&self, pos: &Position) -> char {
        self.0[pos.y][pos.x]
    }
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut chars = Vec::new();

        for line in self.0.iter() {
            for c in line.iter() {
                chars.push(c.to_string());
            }
            chars.push("\n".to_string());
        }

        write!(f, "{}", chars.join(""))
    }
}

#[derive(Debug, Copy, Clone)]
struct Position {
    x: usize,
    y: usize,
}

#[derive(Debug, Copy, Clone, Rand)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}


#[derive(Debug, Copy, Clone)]
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

fn run(program: Program) -> u64 {
    let mut pointer = InstructionPointer::new();
    let mut stack = Stack::new();

    let mut rng = rand::thread_rng();

    let mut instruction_count = 0;

    loop {
        instruction_count += 1;

//        println!("{:?} : {:?}", program.get(&pointer.position), stack);

        // execute instruction at pointer
        // https://esolangs.org/wiki/Befunge#Instructions
        match program.get(&pointer.position) {
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
                stack.push(a.clone());
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
            '@' => break,
            c @ '0'...'9' => stack.push(c.to_digit(10).unwrap_or(0) as i64),
            _ => {}
        }

        // move pointer
        match pointer.direction {
            Direction::Up => { pointer.position.y -= 1 }
            Direction::Down => { pointer.position.y += 1 }
            Direction::Right => { pointer.position.x += 1 }
            Direction::Left => { pointer.position.x -= 1 }
        }
    }

    instruction_count
}

fn benchmark(program: Program) {
    let start = PreciseTime::now();
    let instruction_count = run(program);
    let end = PreciseTime::now();

    let duration = start.to(end);
    println!("\n");
    println!("Executed {:?} instructions in {:?} μs", instruction_count, duration.num_microseconds().unwrap());
    let num_seconds = 1.0e-9 * duration.num_nanoseconds().unwrap() as f64;
    println!("Running at {:.0} instructions/second", instruction_count as f64 / num_seconds);
}
