use std::{fmt, fs::File, io, io::Read, str::FromStr};

#[derive(Copy, Clone)]
pub struct Program(pub [[char; 80]; 30]);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl Program {
    fn new() -> Self {
        Program([[' '; 80]; 30])
    }

    pub fn from_file(path: &str) -> Result<Self, io::Error> {
        let mut f = File::open(path)?;
        let mut contents = String::new();
        f.read_to_string(&mut contents)?;

        Program::from_str(&contents)
    }

    pub fn get(&self, pos: &Position) -> char {
        self.0[pos.y][pos.x]
    }

    pub fn set(&mut self, pos: &Position, c: char) {
        self.0[pos.y][pos.x] = c;
    }

    pub fn show(&self) {
        println!("{}", self);
    }
}

impl FromStr for Program {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Program, io::Error> {
        let mut program = Program::new();

        for (y, line) in s.lines().enumerate() {
            for (x, ch) in line.chars().enumerate() {
                program.set(&Position { x, y }, ch);
            }
        }

        Ok(program)
    }
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut chars = Vec::new();
        let bar = vec!["─"; 80].join("");
        chars.push(format!("┌{}┐\n", bar));
        for line in &self.0 {
            chars.push("│".to_string());
            for c in line.iter() {
                chars.push(c.to_string());
            }
            chars.push("│".to_string());
            chars.push('\n'.to_string());
        }
        chars.push(format!("└{}┘", bar));

        write!(f, "{}", chars.join(""))
    }
}
