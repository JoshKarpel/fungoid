use std::str::FromStr;
use std::{
    cmp::Ordering,
    fmt,
    fs::File,
    io::{self, prelude::*},
    time::Duration,
    time::Instant,
};

use crossterm::event::{poll, DisableMouseCapture, EnableMouseCapture, Event, KeyCode};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{event, execute};
use humantime::format_duration;
use rand::{
    distributions::{Distribution, Standard},
    prelude::*,
    Rng,
};
use separator::Separatable;
use tui::backend::{Backend, CrosstermBackend};
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Style};
use tui::text::Span;
use tui::widgets::canvas::Canvas;
use tui::widgets::{Block, Borders};
use tui::{Frame, Terminal};

#[derive(Copy, Clone)]
pub struct Program([[char; 80]; 30]);

impl FromStr for Program {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Program, io::Error> {
        let mut program = Program::new();

        for (y, line) in s.split('\n').enumerate() {
            for (x, ch) in line.chars().enumerate() {
                program.set(&Position { x, y }, ch);
            }
        }

        Ok(program)
    }
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
enum PointerDirection {
    Up,
    Down,
    Left,
    Right,
}

impl Distribution<PointerDirection> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> PointerDirection {
        match rng.gen_range(0..4) {
            0 => PointerDirection::Up,
            1 => PointerDirection::Down,
            2 => PointerDirection::Left,
            _ => PointerDirection::Right,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct InstructionPointer {
    position: Position,
    direction: PointerDirection,
}

impl InstructionPointer {
    fn new() -> Self {
        InstructionPointer {
            position: Position { x: 0, y: 0 },
            direction: PointerDirection::Right,
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
            self.step();
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

    fn step(&mut self) {
        if self.trace {
            self.trace();
        }

        // execute instruction at pointer
        // https://esolangs.org/wiki/Befunge#Instructions
        match self.program.get(&self.pointer.position) {
            '"' => self.string_mode = !self.string_mode,
            c if self.string_mode => self.stack.push(i64::from(c as u8)),
            '^' => self.pointer.direction = PointerDirection::Up,
            'v' => self.pointer.direction = PointerDirection::Down,
            '>' => self.pointer.direction = PointerDirection::Right,
            '<' => self.pointer.direction = PointerDirection::Left,
            '?' => self.pointer.direction = self.rng.gen(),
            '_' => {
                // horizontal if
                let top = self.stack.pop();
                if top == 0 {
                    self.pointer.direction = PointerDirection::Right;
                } else {
                    self.pointer.direction = PointerDirection::Left;
                }
            }
            // vertical if
            '|' => {
                let top = self.stack.pop();
                if top == 0 {
                    self.pointer.direction = PointerDirection::Down;
                } else {
                    self.pointer.direction = PointerDirection::Up;
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
            }
            ',' => {
                write!(self.output, "{}", self.stack.pop() as u8 as char)
                    .expect("Failed to write char");
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
                return;
            }
            c @ '0'..='9' => self.stack.push(i64::from(c.to_digit(10).unwrap())),
            ' ' => {}
            c => panic!("Unrecognized instruction! {}", c),
        }

        move_pointer(&mut self.pointer);

        self.instruction_count += 1;
    }
}

fn move_pointer(pointer: &mut InstructionPointer) {
    match pointer.direction {
        PointerDirection::Up => pointer.position.y -= 1,
        PointerDirection::Down => pointer.position.y += 1,
        PointerDirection::Right => pointer.position.x += 1,
        PointerDirection::Left => pointer.position.x -= 1,
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
        format_duration(duration),
        ((instruction_count as f64 / num_seconds) as u64).separated_string()
    );
}

pub fn step(program: Program, delay: Duration) -> crossterm::Result<()> {
    let mut stdin = io::stdin();
    let mut output = Vec::new();
    let mut program_state = ProgramState::new(program, false, &mut stdin, &mut output);

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let res = run_app(&mut terminal, &mut program_state, delay);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend, O: Write>(
    terminal: &mut Terminal<B>,
    program_state: &mut ProgramState<O>,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    let mut paused = false;
    loop {
        terminal.draw(|f| ui(f, program_state))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
                    KeyCode::Char(' ') => paused = !paused,
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            if !paused {
                program_state.step();
            }
            last_tick = Instant::now();
        }
    }
}

fn ui<B: Backend, O: Write>(f: &mut Frame<B>, program_state: &ProgramState<O>) {
    let w = f.size().width as f64;
    let h = f.size().height as f64;

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(f.size());
    let canvas = Canvas::default()
        .block(Block::default().borders(Borders::ALL).title("Program"))
        .paint(|ctx| {
            for (y, line) in program_state.program.0.iter().enumerate() {
                for (x, c) in line.iter().enumerate() {
                    let style = if program_state.pointer.position.x == x
                        && program_state.pointer.position.y == y
                    {
                        Style::default().bg(Color::Green)
                    } else {
                        Style::default().fg(Color::White)
                    };

                    ctx.print(
                        x as f64,
                        h - (y as f64) - 1.0,
                        Span::styled(c.to_string(), style),
                    );
                }
            }
        })
        .x_bounds([0.0, w])
        .y_bounds([0.0, h]);
    f.render_widget(canvas, chunks[0]);
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::str::FromStr;

    use crate::{Program, ProgramState};

    const HELLO_WORLD: &'static str = r#"64+"!dlroW ,olleH">:#,_@"#;

    #[test]
    fn hello_world() -> Result<(), io::Error> {
        let program = Program::from_str(HELLO_WORLD)?;
        println!("{}", program);
        let mut output = Vec::new();
        ProgramState::new(program, false, &mut io::stdin(), &mut output).run();
        println!("{:?}", output);
        assert_eq!("Hello, World!\n", String::from_utf8(output).unwrap());

        Ok(())
    }

    const SIEVE_OF_ERATOSTHENES: &'static str = r#"2>:3g" "-!v\  g30          <
 |!`"O":+1_:.:03p>03g+:"O"`|
 @               ^  p3\" ":<
2 234567890123456789012345678901234567890123456789012345678901234567890123456789
"#;

    #[test]
    fn sieve_of_eratosthenes() -> Result<(), io::Error> {
        let program = Program::from_str(SIEVE_OF_ERATOSTHENES)?;
        println!("{}", program);
        let mut output = Vec::new();
        ProgramState::new(program, false, &mut io::stdin(), &mut output).run();
        println!("{:?}", output);
        assert_eq!(
            "2357111317192329313741434753596167717379",
            String::from_utf8(output).unwrap()
        );

        Ok(())
    }

    const QUINE: &'static str = r#"01->1# +# :# 0# g# ,# :# 5# 8# *# 4# +# -# _@"#;

    #[test]
    fn quine() -> Result<(), io::Error> {
        let program = Program::from_str(QUINE)?;
        println!("{}", program);
        let mut output = Vec::new();
        ProgramState::new(program, false, &mut io::stdin(), &mut output).run();
        println!("{:?}", output);
        assert_eq!(
            "01->1# +# :# 0# g# ,# :# 5# 8# *# 4# +# -# _@",
            String::from_utf8(output).unwrap()
        );

        Ok(())
    }
}
