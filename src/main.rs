extern crate rand;
#[macro_use]
extern crate rand_derive;


use std::fmt;
use std::cmp::Ordering;

use rand::Rng;

fn main() {
    let program = Program::from_str("0956+++.@");
//    let program = Program::from_str(&vec![r#"5>:1-:v v *_$.@ "#, r#" ^    _$>\:^"#].join("\n"));
//    let program = Program::from_str(r#"64+"!dlroW ,olleH">:#,_@"#);

    println!("PROGRAM");
    println!("{}", vec!["-"; 80].join(""));
    println!("{}", program);
    println!("{}", vec!["-"; 80].join(""));

    println!("OUTPUT");
    run(program);
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

struct Stack(Vec<usize>);

impl Stack {
    fn new() -> Stack {
        Stack(Vec::<usize>::new())
    }
    fn push(&mut self, val: usize) {
        self.0.push(val);
    }

    fn pop(&mut self) -> usize {
        self.0.pop().unwrap_or(0)
    }
}

fn run(program: Program) {
    let mut pointer = InstructionPointer::new();
    let mut stack = Stack::new();

    let mut rng = rand::thread_rng();

    loop {
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
                stack.push(b);
                stack.push(a);
            }
            '$' => { // discard top of stack
                stack.pop();
            }
            '.' => print!("{}", stack.pop()),
            '@' => break,
            c @ '0'...'9' => stack.push(c.to_digit(10).unwrap_or(0) as usize),
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
}
