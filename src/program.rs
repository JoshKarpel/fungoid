use std::collections::HashMap;
use std::{fs::File, io, io::Read, str::FromStr};

use itertools::{Itertools, MinMaxResult};

#[derive(Debug, Clone)]
pub struct Program(HashMap<Position, char>);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Position {
    pub x: isize,
    pub y: isize,
}

impl Program {
    fn new() -> Self {
        Program(HashMap::new())
    }

    pub fn get(&self, pos: &Position) -> char {
        *self.0.get(pos).unwrap_or(&' ')
    }

    pub fn set(&mut self, pos: &Position, c: char) {
        self.0.insert(*pos, c);
    }

    pub fn from_file(path: &str) -> Result<Self, io::Error> {
        let mut f = File::open(path)?;
        let mut contents = String::new();
        f.read_to_string(&mut contents)?;

        Program::from_str(&contents)
    }

    pub fn chars(&self) -> Vec<(Position, char)> {
        match self.0.keys().minmax() {
            MinMaxResult::NoElements => vec![],
            MinMaxResult::OneElement(p) => vec![(*p, self.get(p))],
            MinMaxResult::MinMax(upper_left, lower_right) => (upper_left.y..=lower_right.y)
                .cartesian_product(upper_left.x..=lower_right.x)
                .map(|(y, x)| {
                    let p = Position { x, y };
                    (p, self.get(&p))
                })
                .collect(),
        }
    }

    pub fn view(&self, upper_left: &Position, lower_right: &Position) -> Vec<(Position, char)> {
        (lower_right.y..=upper_left.y)
            .rev()
            .cartesian_product(upper_left.x..=lower_right.x)
            .map(move |(y, x)| {
                let p = Position { x, y };
                (p, self.get(&p))
            })
            .collect_vec()
    }
}

impl FromStr for Program {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Program, io::Error> {
        let mut program = Program::new();

        for (y, line) in s.lines().enumerate() {
            for (x, c) in line.chars().enumerate() {
                program.set(
                    &Position {
                        x: x as isize,
                        y: -(y as isize),
                    },
                    c,
                );
            }
        }

        Ok(program)
    }
}
