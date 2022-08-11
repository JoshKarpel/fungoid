use std::{
    fmt::Write,
    io,
    time::{Duration, Instant},
};

use crossterm::{
    event,
    event::{poll, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use itertools::Itertools;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Cell, List, ListItem, Paragraph, Row, Table, Wrap},
    Frame, Terminal,
};

use crate::{
    execution::{ExecutionError, ExecutionState},
    ide::HandleKeyResult::{Continue, Quit},
    program::{Position, Program},
};

pub fn ide(program: Program) -> crossterm::Result<()> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let res = run_ide(&mut terminal, program);

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

struct IDEState {
    instructions_per_second: usize,
    paused: bool,
    following: bool,
    editing: bool,
    view_center: Position,
    error: Option<ExecutionError>,
}

impl IDEState {
    fn new() -> Self {
        IDEState {
            instructions_per_second: 10,
            paused: true,
            following: false,
            editing: false,
            view_center: Position { x: 0, y: 0 },
            error: None,
        }
    }

    fn tick_time(&self) -> Duration {
        Duration::from_secs_f64(1.0 / (self.instructions_per_second as f64))
    }
}

fn run_ide<B: Backend>(terminal: &mut Terminal<B>, mut program: Program) -> io::Result<()> {
    let input = Vec::new();
    let output = Vec::new();
    let mut execution_state = ExecutionState::new(program.clone(), false, input.as_slice(), output);

    let mut last_tick = Instant::now();

    let mut ide_state = IDEState::new();

    loop {
        terminal.draw(|f| ui(f, &execution_state, &ide_state))?;

        let tick_time = ide_state.tick_time();

        let time_to_next_tick = tick_time
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if poll(time_to_next_tick)? {
            if let Quit = handle_key(
                event::read()?,
                &mut ide_state,
                &mut execution_state,
                &mut program,
            ) {
                return Ok(());
            }
        }

        // When handling input, we might not wait the whole poll() above, so check to see if we should tick.
        if last_tick.elapsed() >= tick_time {
            if let Quit = handle_tick(&mut ide_state, &mut execution_state, &mut program) {
                return Ok(());
            }
            last_tick = Instant::now();
        }
    }
}

enum HandleKeyResult {
    Continue,
    Quit,
}

fn handle_key(
    event: Event,
    ide_state: &mut IDEState,
    execution_state: &mut ExecutionState<&[u8], Vec<u8>>,
    program: &mut Program,
) -> HandleKeyResult {
    if let Event::Key(key) = event {
        match key.code {
            KeyCode::Char('i') if !ide_state.editing => {
                ide_state.paused = true;
                ide_state.editing = true;

                execution_state.reset();
                execution_state.program = program.clone();
                execution_state.output.clear();
            }
            KeyCode::Esc if ide_state.editing => {
                ide_state.editing = false;
            }
            KeyCode::Char(c) if ide_state.editing => {
                program.set(&ide_state.view_center, c);
                execution_state.program = program.clone();
            }
            KeyCode::Char('q') => {
                return Quit;
            }
            KeyCode::Char('r') => {
                // TODO: figure out a cleaner way to handle "execute from scratch"
                execution_state.reset();
                execution_state.program = program.clone();
                execution_state.output.clear();
            }
            KeyCode::Char(' ') if !ide_state.editing => ide_state.paused = !ide_state.paused,
            KeyCode::Char('t') if !ide_state.editing => {
                ide_state.paused = true;
                let result = execution_state.step();
                if let Err(e) = result {
                    ide_state.error = Some(e);
                    execution_state.reset();
                    execution_state.program = program.clone();
                    execution_state.output.clear();
                }
            }
            KeyCode::Char('f') => ide_state.following = !ide_state.following,
            KeyCode::Char('+') => {
                ide_state.instructions_per_second = (ide_state.instructions_per_second + 1).max(1)
            }
            KeyCode::Char('-') => {
                ide_state.instructions_per_second = (ide_state.instructions_per_second - 1).max(1)
            }
            KeyCode::Left => {
                ide_state.view_center = ide_state.view_center.shifted(-1, 0);
                ide_state.following = false;
            }
            KeyCode::Right => {
                ide_state.view_center = ide_state.view_center.shifted(1, 0);
                ide_state.following = false;
            }
            KeyCode::Up => {
                ide_state.view_center = ide_state.view_center.shifted(0, -1);
                ide_state.following = false;
            }
            KeyCode::Down => {
                ide_state.view_center = ide_state.view_center.shifted(0, 1);
                ide_state.following = false;
            }
            _ => {}
        }
    }

    Continue
}

fn handle_tick(
    ide_state: &mut IDEState,
    execution_state: &mut ExecutionState<&[u8], Vec<u8>>,
    program: &mut Program,
) -> HandleKeyResult {
    if !ide_state.paused {
        let result = execution_state.step();

        if ide_state.following {
            ide_state.view_center = execution_state.pointer.position
        }

        if let Err(e) = result {
            ide_state.paused = true;
            ide_state.error = Some(e);
            execution_state.reset();
            execution_state.program = program.clone();
            execution_state.output.clear();
        }
    }

    Continue
}

fn ui<B: Backend>(
    f: &mut Frame<B>,
    program_state: &ExecutionState<&[u8], Vec<u8>>,
    ide_state: &IDEState,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
        .split(f.size());
    let upper = chunks[0];
    let lower = chunks[1];

    let upper_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(90), Constraint::Percentage(10)].as_ref())
        .split(upper);
    let program_area = upper_chunks[0];
    let stack_area = upper_chunks[1];

    let lower_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(90), Constraint::Percentage(10)].as_ref())
        .split(lower);
    let output_area = lower_chunks[0];
    let state_area = lower_chunks[1];

    let w = program_area.width as isize;
    let h = program_area.height as isize;

    let upper_left = Position {
        x: ide_state.view_center.x - w / 2,
        y: ide_state.view_center.y - h / 2,
    };
    let lower_right = Position {
        x: upper_left.x + w,
        y: upper_left.y + h,
    };

    let widths = vec![Constraint::Length(1); w as usize];

    let program_grid = Table::new(
        program_state
            .program
            .view(&upper_left, &lower_right)
            .group_by(|(p, _)| p.y)
            .into_iter()
            .map(|(_, row)| {
                Row::new(row.map(|(p, c)| {
                    let style = if p == program_state.pointer.position {
                        if program_state.terminated {
                            Style::default().bg(Color::Red)
                        } else {
                            Style::default().bg(Color::Green)
                        }
                    } else if p == ide_state.view_center {
                        Style::default().bg(Color::LightMagenta)
                    } else {
                        Style::default()
                    };
                    Cell::from(c.to_string()).style(style)
                }))
            }),
    )
    .block(
        Block::default()
            .title(format!(
                " Program | (x, y) = ({}, {}) ",
                ide_state.view_center.x, ide_state.view_center.y
            ))
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL),
    )
    .style(Style::default().fg(Color::White).bg(Color::Black))
    .widths(&*widths)
    .column_spacing(0);

    let stack = List::new(
        program_state
            .stack
            .items()
            .iter()
            .map(|i| ListItem::new(i.to_string()))
            .collect_vec(),
    )
    .block(
        Block::default()
            .title(" Stack ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL),
    )
    .style(Style::default().fg(Color::White));

    let mut o = std::str::from_utf8(&program_state.output)
        .unwrap()
        .to_string();
    if let Some(e) = &ide_state.error {
        write!(o, "\n{}", e).expect("Failed to generate error message");
    }
    let output = Paragraph::new(o)
        .block(
            Block::default()
                .title(" Output ")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(Color::White).bg(Color::Black))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: false });

    let mut settings = vec![];
    if ide_state.editing {
        settings.push(ListItem::new("editing"));
    }
    if ide_state.paused {
        settings.push(ListItem::new("paused"));
    }
    if ide_state.following {
        settings.push(ListItem::new("following"));
    }
    let state = List::new(settings)
        .block(
            Block::default()
                .title(" IDE ")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(Color::White));

    f.render_widget(program_grid, program_area);
    f.render_widget(stack, stack_area);
    f.render_widget(output, output_area);
    f.render_widget(state, state_area);
}
