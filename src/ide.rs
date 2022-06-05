use std::{
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
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap},
    widgets::{List, ListItem},
    Frame, Terminal,
};

use crate::execution::ExecutionState;
use crate::program::Program;
use crate::Position;

pub fn ide(program: Program) -> crossterm::Result<()> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let res = run_app(&mut terminal, program);

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
    paused: bool,
    follow: bool,
    view_center: Position,
}

impl IDEState {
    fn new() -> Self {
        IDEState {
            paused: false,
            follow: false,
            view_center: Position { x: 0, y: 0 },
        }
    }
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, program: Program) -> io::Result<()> {
    let mut max_ips: u32 = 10;

    let mut stdin = io::stdin();
    let mut output = Vec::new();

    let mut program_state = ExecutionState::new(program, false, &mut stdin, &mut output);

    let mut last_tick = Instant::now();

    let mut ide_state = IDEState::new();

    loop {
        terminal.draw(|f| ui(f, &program_state, &ide_state))?;

        let tick_rate = Duration::from_secs_f64(1.0 / (max_ips as f64));

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
                    KeyCode::Char('r') => {
                        program_state.reset();
                        program_state.output.clear();
                    }
                    KeyCode::Char(' ') => ide_state.paused = !ide_state.paused,
                    KeyCode::Char('t') => {
                        ide_state.paused = true;
                        program_state.step();
                    }
                    KeyCode::Char('f') => ide_state.follow = !ide_state.follow,
                    KeyCode::Char('+') => max_ips = (max_ips + 1).max(1),
                    KeyCode::Char('-') => max_ips = (max_ips - 1).max(1),
                    KeyCode::Left => {
                        ide_state.view_center = Position {
                            x: ide_state.view_center.x - 1,
                            y: ide_state.view_center.y,
                        };
                        ide_state.follow = false;
                    }
                    KeyCode::Right => {
                        ide_state.view_center = Position {
                            x: ide_state.view_center.x + 1,
                            y: ide_state.view_center.y,
                        };
                        ide_state.follow = false;
                    }
                    KeyCode::Up => {
                        ide_state.view_center = Position {
                            x: ide_state.view_center.x,
                            y: ide_state.view_center.y - 1,
                        };
                        ide_state.follow = false;
                    }
                    KeyCode::Down => {
                        ide_state.view_center = Position {
                            x: ide_state.view_center.x,
                            y: ide_state.view_center.y + 1,
                        };
                        ide_state.follow = false;
                    }
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            if !ide_state.paused {
                program_state.step();
                if ide_state.follow {
                    ide_state.view_center = program_state.pointer.position
                }
            }
            last_tick = Instant::now();
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, program_state: &ExecutionState<Vec<u8>>, ide_state: &IDEState) {
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

    let pointer_position = program_state.pointer.position;
    let terminated = program_state.terminated;

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
            .iter()
            .group_by(|(p, _)| p.y)
            .into_iter()
            .map(|(_, row)| {
                Row::new(row.map(|(p, c)| {
                    let style = if p == &pointer_position {
                        if terminated {
                            Style::default().bg(Color::Red)
                        } else {
                            Style::default().bg(Color::Green)
                        }
                    } else if p == &ide_state.view_center {
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

    let o = std::str::from_utf8(program_state.output).unwrap();
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
    if ide_state.paused {
        settings.push(ListItem::new("paused"));
    }
    if ide_state.follow {
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
