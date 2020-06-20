extern crate crossterm;
extern crate humantime;
extern crate rand;
extern crate separator;

use std::{
    cmp::Ordering,
    fmt,
    fs::File,
    io::{self, prelude::*},
    time::Duration,
    time::Instant,
};

use crossterm::{
    cursor,
    event::{poll, read, Event, KeyCode, KeyEvent, KeyModifiers},
    style::{self, Colorize},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand, QueueableCommand, Result,
};
use humantime::format_duration;
use rand::{
    distributions::{Distribution, Standard},
    prelude::*,
    Rng,
};
use separator::Separatable;

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

    pub fn str(&self) -> String {
        format!("{}", self)
    }

    pub fn show(&self) {
        println!("{}", self);
    }

    pub fn draw(&self, mut stdout: &mut dyn Write) -> Result<()> {
        let bar = vec!["─"; 80].join("");
        let q = stdout
            .queue(cursor::MoveTo(0, 0))?
            .queue(style::PrintStyledContent(format!("┌{}┐", bar).white()))?;
        for line in &self.0 {
            q.queue(cursor::MoveToNextLine(1))?
                .queue(style::PrintStyledContent(
                    format!("│{}│", line.iter().collect::<String>()).white(),
                ))?;
        }
        q.queue(cursor::MoveToNextLine(1))?
            .queue(style::PrintStyledContent(format!("└{}┘", bar).white()))?;
        stdout.flush()?;
        Ok(())
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
    fn new() -> Self {
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
    terminated: bool,
    string_mode: bool,
    instruction_count: u64,
    input: &'input mut dyn Read,
    output: &'output mut dyn Write,
    error: &'error mut dyn Write,
}

impl<'input, 'output, 'error> ProgramState<'input, 'output, 'error> {
    pub fn new(
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
            terminated: false,
            string_mode: false,
            instruction_count: 0,
            input,
            output,
            error,
        }
    }

    fn run(mut self) -> Self {
        while !self.terminated {
            self = self.step();
        }

        self
    }

    fn step(mut self) -> Self {
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
            '@' => {
                self.terminated = true;
                self.instruction_count += 1;
                return self;
            }
            c @ '0'..='9' => self.stack.push(i64::from(c.to_digit(10).unwrap())),
            ' ' => {}
            c => panic!("Unrecognized instruction! {}", c),
        }

        move_pointer(&mut self.pointer);

        self.instruction_count += 1;
        self
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

pub fn run_to_termination(program_state: ProgramState) -> u64 {
    program_state.run().instruction_count
}

pub fn time(program_state: ProgramState) {
    let start = Instant::now();
    let instruction_count = run_to_termination(program_state);
    let duration = start.elapsed();

    let num_seconds = 1.0e-9 * (duration.as_nanos() as f64);

    eprintln!(
        "Executed {} instructions in {} ({} instructions/second)",
        instruction_count,
        format_duration(duration).to_string(),
        ((instruction_count as f64 / num_seconds) as u64).separated_string()
    );
}

pub fn step(program: Program, delay: Duration) -> Result<()> {
    let mut stdout = io::stdout();
    let mut stdin = io::stdin();
    let mut stderr = io::stderr();

    let mut streams = StepStreams::new(&mut stdin, &mut stdout, &mut stderr);

    let mut input = io::stdin();
    let mut output = Vec::new();
    let mut error = Vec::new();
    let mut program_state = ProgramState::new(program, &mut input, &mut output, &mut error);

    streams.output.execute(EnterAlternateScreen)?;

    enable_raw_mode()?;

    program.draw(&mut streams.output)?;

    while !program_state.terminated {
        streams
            .output
            .queue(cursor::MoveTo(
                (program_state.pointer.position.x + 1) as u16,
                (program_state.pointer.position.y + 1) as u16,
            ))?
            .queue(style::PrintStyledContent(
                program.get(&program_state.pointer.position).white(),
            ))?;

        program_state = program_state.step();

        streams
            .output
            .queue(cursor::MoveTo(
                (program_state.pointer.position.x + 1) as u16,
                (program_state.pointer.position.y + 1) as u16,
            ))?
            .queue(style::PrintStyledContent(
                program.get(&program_state.pointer.position).red(),
            ))?;

        streams.output.flush()?;

        if poll(delay)? {
            match read()? {
                Event::Key(KeyEvent {
                    modifiers: KeyModifiers::CONTROL,
                    code: KeyCode::Char('c'),
                }) => break,
                _ => {}
            }
        }
    }

    streams.output.execute(LeaveAlternateScreen)?;

    disable_raw_mode()?;

    Ok(())
}

pub struct StepStreams<'input, 'output, 'error> {
    input: &'input mut dyn Read,
    output: &'output mut dyn Write,
    error: &'error mut dyn Write,
}

impl<'input, 'output, 'error> StepStreams<'input, 'output, 'error> {
    fn new(
        input: &'input mut dyn Read,
        output: &'output mut dyn Write,
        error: &'error mut dyn Write,
    ) -> Self {
        StepStreams {
            input,
            output,
            error,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Program, ProgramState};

    use std::io;

    const HELLO_WORLD: &'static str = r#"64+"!dlroW ,olleH">:#,_@"#;

    #[test]
    fn hello_world() {
        let program = Program::from_str(HELLO_WORLD);
        println!("{}", program);
        let mut output = Vec::new();
        ProgramState::new(program, &mut io::stdin(), &mut output, &mut Vec::new()).run();
        println!("{:?}", output);
        assert_eq!("Hello, World!\n", String::from_utf8(output).unwrap());
    }
}
