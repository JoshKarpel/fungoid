extern crate chrono;
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

use crossterm::cursor::{MoveToNextLine, RestorePosition, SavePosition};
use crossterm::{
    cursor,
    event::{poll, read, Event, KeyCode, KeyEvent, KeyModifiers},
    style::{self, Colorize},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand, QueueableCommand,
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
    fn new() -> Self {
        Program([[' '; 80]; 30])
    }

    pub fn from_str(s: &str) -> Self {
        let mut program = Program::new();

        for (y, line) in s.split('\n').enumerate() {
            for (x, ch) in line.chars().enumerate() {
                program.set(&Position { x, y }, ch);
            }
        }

        program
    }

    pub fn from_file(path: &str) -> Result<Self, std::io::Error> {
        let mut f = File::open(path)?;
        let mut contents = String::new();
        f.read_to_string(&mut contents)?;

        Ok(Program::from_str(&contents))
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

    pub fn draw(&self, mut stdout: &mut dyn Write) -> crossterm::Result<()> {
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

    fn join(&self, sep: &str) -> String {
        return self
            .0
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(sep);
    }
}

pub struct ProgramState<'input, 'output, O: Write> {
    program: Program,
    pointer: InstructionPointer,
    stack: Stack,
    rng: ThreadRng,
    terminated: bool,
    string_mode: bool,
    trace: bool,
    instruction_count: u64,
    input: &'input mut dyn Read,
    output: &'output mut O,
}

impl<'input, 'output, O: Write> ProgramState<'input, 'output, O> {
    pub fn new(
        program: Program,
        trace: bool,
        input: &'input mut dyn Read,
        output: &'output mut O,
    ) -> Self {
        ProgramState {
            program,
            pointer: InstructionPointer::new(),
            stack: Stack::new(),
            rng: rand::thread_rng(),
            terminated: false,
            string_mode: false,
            trace,
            instruction_count: 0,
            input,
            output,
        }
    }

    fn run(mut self) -> Self {
        while !self.terminated {
            self = self.step();
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

    fn step(mut self) -> Self {
        if self.trace {
            self.trace();
        }

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

impl<'input, 'output> ProgramState<'input, 'output, Vec<u8>> {
    fn pop_output(&mut self) -> Option<u8> {
        self.output.pop()
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

pub fn run_to_termination<O: Write>(program_state: ProgramState<O>) -> u64 {
    program_state.run().instruction_count
}

pub fn time<O: Write>(program_state: ProgramState<O>) {
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

pub fn step(program: Program, delay: Duration) -> crossterm::Result<()> {
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();
    let mut streams = StepStreams::new(&mut stdin, &mut stdout, &mut stderr);
    streams.init()?;

    let mut input = io::stdin();
    let mut output = Vec::new();
    let mut program_state = ProgramState::new(program, false, &mut input, &mut output);

    program.draw(&mut streams.output)?;

    streams
        .output
        .queue(MoveToNextLine(2))?
        .execute(SavePosition)?;

    let mut output_width: u16 = 0;
    let (terminal_width, _terminal_rows) = crossterm::terminal::size().unwrap_or((80, 30));

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
                program.get(&program_state.pointer.position).green(),
            ))?;

        if let Some(c) = program_state.pop_output() {
            output_width += 1;
            match c {
                // newline
                _ if output_width > terminal_width - 2 => {
                    output_width = 0;
                    streams
                        .output
                        .queue(RestorePosition)?
                        .queue(style::PrintStyledContent("⏎".green()))?
                        .queue(MoveToNextLine(1))?
                        .queue(SavePosition)?
                }
                10 => {
                    output_width = 0;
                    streams
                        .output
                        .queue(RestorePosition)?
                        .queue(MoveToNextLine(1))?
                        .queue(SavePosition)?
                }
                _ => streams
                    .output
                    .queue(RestorePosition)?
                    .queue(style::PrintStyledContent((c as char).white()))?
                    .queue(SavePosition)?,
            };
        };

        streams.output.flush()?;

        if poll(delay)? {
            match read()? {
                Event::Key(KeyEvent {
                    modifiers: KeyModifiers::CONTROL,
                    code: KeyCode::Char('c'),
                }) => return Ok(()),
                _ => {}
            }
        }
    }

    streams
        .output
        .queue(cursor::MoveTo(
            (program_state.pointer.position.x + 1) as u16,
            (program_state.pointer.position.y + 1) as u16,
        ))?
        .execute(style::PrintStyledContent(
            program.get(&program_state.pointer.position).red(),
        ))?;

    loop {
        match read()? {
            Event::Key(KeyEvent {
                modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Char('c'),
            }) => return Ok(()),
            _ => {}
        }
    }
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

    fn init(&mut self) -> crossterm::Result<()> {
        self.output.execute(EnterAlternateScreen)?;
        enable_raw_mode()?;
        Ok(())
    }
}

impl<'input, 'output, 'error> Drop for StepStreams<'input, 'output, 'error> {
    fn drop(&mut self) {
        self.output.execute(LeaveAlternateScreen).unwrap();
        disable_raw_mode().unwrap();
        ()
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
        ProgramState::new(program, false, &mut io::stdin(), &mut output).run();
        println!("{:?}", output);
        assert_eq!("Hello, World!\n", String::from_utf8(output).unwrap());
    }

    const SIEVE_OF_ERATOSTHENES: &'static str = r#"2>:3g" "-!v\  g30          <
 |!`"O":+1_:.:03p>03g+:"O"`|
 @               ^  p3\" ":<
2 234567890123456789012345678901234567890123456789012345678901234567890123456789
"#;

    #[test]
    fn sieve_of_eratosthenes() {
        let program = Program::from_str(SIEVE_OF_ERATOSTHENES);
        println!("{}", program);
        let mut output = Vec::new();
        ProgramState::new(program, false, &mut io::stdin(), &mut output).run();
        println!("{:?}", output);
        assert_eq!(
            "2357111317192329313741434753596167717379",
            String::from_utf8(output).unwrap()
        );
    }

    const QUINE: &'static str = r#"01->1# +# :# 0# g# ,# :# 5# 8# *# 4# +# -# _@"#;

    #[test]
    fn quine() {
        let program = Program::from_str(QUINE);
        println!("{}", program);
        let mut output = Vec::new();
        ProgramState::new(program, false, &mut io::stdin(), &mut output).run();
        println!("{:?}", output);
        assert_eq!(
            "01->1# +# :# 0# g# ,# :# 5# 8# *# 4# +# -# _@",
            String::from_utf8(output).unwrap()
        );
    }
}
