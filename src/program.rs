use std::{collections::HashMap, fs::File, io, io::Read, str::FromStr};

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

    pub fn view(&self, upper_left: &Position, lower_right: &Position) -> Vec<(Position, char)> {
        (upper_left.y..=lower_right.y)
            .cartesian_product(upper_left.x..=lower_right.x)
            .map(move |(y, x)| {
                let p = Position { x, y };
                (p, self.get(&p))
            })
            .collect_vec()
    }

    pub fn bounds(&self) -> Option<(Position, Position)> {
        match self.0.keys().minmax() {
            MinMaxResult::NoElements => None,
            MinMaxResult::OneElement(e) => Some((*e, *e)),
            MinMaxResult::MinMax(ul, lr) => Some((*ul, *lr)),
        }
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
                        y: y as isize,
                    },
                    c,
                );
            }
        }

        Ok(program)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::program::{Position, Program};

    pub type GenericResult = Result<(), Box<dyn std::error::Error>>;

    #[test]
    fn test_from_str() -> GenericResult {
        let program = Program::from_str("12\n34")?;

        assert_eq!(program.get(&Position { x: 0, y: 0 }), '1');
        assert_eq!(program.get(&Position { x: 1, y: 0 }), '2');
        assert_eq!(program.get(&Position { x: 0, y: 1 }), '3');
        assert_eq!(program.get(&Position { x: 1, y: 1 }), '4');

        Ok(())
    }

    #[test]
    fn test_can_set_and_get_a_cell() -> GenericResult {
        let mut program = Program::new();
        program.set(&Position { x: 0, y: 0 }, '.');
        assert_eq!(program.get(&Position { x: 0, y: 0 }), '.');

        Ok(())
    }

    #[test]
    fn test_bounds_with_empty_program_is_none() -> GenericResult {
        let program = Program::new();

        assert!(program.bounds().is_none());

        Ok(())
    }

    #[test]
    fn test_bounds_with_one_cell_program_is_that_cell_twice() -> GenericResult {
        let mut program = Program::new();
        program.set(&Position { x: 0, y: 0 }, '.');

        assert_eq!(
            program.bounds().unwrap(),
            (Position { x: 0, y: 0 }, Position { x: 0, y: 0 })
        );

        Ok(())
    }

    #[test]
    fn test_bounds_with_multi_cell_program() -> GenericResult {
        let mut program = Program::new();
        program.set(&Position { x: 0, y: 0 }, '.');
        program.set(&Position { x: 1, y: 2 }, '.');

        assert_eq!(
            program.bounds().unwrap(),
            (Position { x: 0, y: 0 }, Position { x: 1, y: 2 })
        );

        Ok(())
    }
}
