use std::fmt;
use std::cmp::Ordering;

fn main() {
//    let program = Program::from_str(r#"64+"!dlroW ,olleH">:#,_@"#);
    let program = Program::from_str("12+.@");

    println!("{}", program);
    println!("{}", vec!["-"; 80].join(""));

    run(program);
}

struct Program([[char; 80]; 30]);


impl Program {
    fn new() -> Program {
        Program([[' '; 80]; 30])
    }

    fn insert(&mut self, pos: &Position, c: char) {
        self.0[pos.y][pos.x] = c;
    }

    fn get(&self, pos: &Position) -> char {
        self.0[pos.y][pos.x]
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
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Copy, Clone)]
struct Position {
    x: usize,
    y: usize,
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

struct BefungeStack(Vec<usize>);

impl BefungeStack {
    fn new() -> BefungeStack {
        BefungeStack(Vec::<usize>::new())
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
    let mut stack = BefungeStack::new();

    loop {
        // https://esolangs.org/wiki/Befunge#Instructions
        match program.get(&pointer.position) {
            '^' => { pointer.direction = Direction::Up }
            'v' => { pointer.direction = Direction::Down }
            '>' => { pointer.direction = Direction::Right }
            '<' => { pointer.direction = Direction::Left }
            '+' => {
                let a = stack.pop();
                let b = stack.pop();
                stack.push(a + b);
            }
            '-' => {
                let a = stack.pop();
                let b = stack.pop();
                stack.push(a - b);
            }
            '*' => {
                let a = stack.pop();
                let b = stack.pop();
                stack.push(a * b);
            }
            '/' => {
                let a = stack.pop();
                let b = stack.pop();
                stack.push(a / b);
            }
            '%' => {
                let a = stack.pop();
                let b = stack.pop();
                stack.push(a % b);
            }
            '!' => {
                let b = stack.pop();
                match b {
                    0 => stack.push(1),
                    _ => stack.push(0),
                }
            }
            '`' => {
                let a = stack.pop();
                let b = stack.pop();
                match b.cmp(&a) {
                    Ordering::Greater => stack.push(1),
                    _ => stack.push(0),
                }
            }
            ':' => {
                let a = stack.pop();
                stack.push(a.clone());
                stack.push(a);
            }
            '\\' => {
                let a = stack.pop();
                let b = stack.pop();
                stack.push(b);
                stack.push(a);
            }
            '$' => {
                stack.pop();
            }
            '.' => { println!("{}", stack.pop()) }
            '@' => {
                println!("fin!");
                break;
            }
            c => { stack.push(c.to_digit(10).unwrap_or(0) as usize) }
        }

        match pointer.direction {
            Direction::Up => { pointer.position.y -= 1 }
            Direction::Down => { pointer.position.y += 1 }
            Direction::Right => { pointer.position.x += 1 }
            Direction::Left => { pointer.position.x -= 1 }
        }
    }
}
