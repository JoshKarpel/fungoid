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

fn run_app<B: Backend>(terminal: &mut Terminal<B>, program: Program) -> io::Result<()> {
    let mut max_ips: u32 = 10;

    let mut stdin = io::stdin();
    let mut output = Vec::new();

    let mut program_state = ExecutionState::new(program, false, &mut stdin, &mut output);

    let mut last_tick = Instant::now();

    let mut paused = false;
    let mut follow = false;
    let mut view_center = Position { x: 0, y: 0 };

    loop {
        terminal.draw(|f| ui(f, &program_state, &view_center))?;

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
                    KeyCode::Char(' ') => paused = !paused,
                    KeyCode::Char('f') => follow = !follow,
                    KeyCode::Char('+') => max_ips = (max_ips + 1).max(1),
                    KeyCode::Char('-') => max_ips = (max_ips - 1).max(1),
                    KeyCode::Left => {
                        view_center = Position {
                            x: view_center.x - 1,
                            y: view_center.y,
                        };
                        follow = false;
                    }
                    KeyCode::Right => {
                        view_center = Position {
                            x: view_center.x + 1,
                            y: view_center.y,
                        };
                        follow = false;
                    }
                    KeyCode::Up => {
                        view_center = Position {
                            x: view_center.x,
                            y: view_center.y + 1,
                        };
                        follow = false;
                    }
                    KeyCode::Down => {
                        view_center = Position {
                            x: view_center.x,
                            y: view_center.y - 1,
                        };
                        follow = false;
                    }
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            if !paused {
                program_state.step();
                if follow {
                    view_center = program_state.pointer.position
                }
            }
            last_tick = Instant::now();
        }
    }
}

fn ui<B: Backend>(
    f: &mut Frame<B>,
    program_state: &ExecutionState<Vec<u8>>,
    view_center: &Position,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
        .split(f.size());

    let pointer_position = program_state.pointer.position;
    let terminated = program_state.terminated;

    let w = chunks[0].width as isize;
    let h = chunks[0].height as isize;

    let upper_left = Position {
        x: view_center.x - w / 2,
        y: view_center.y + h / 2,
    };
    let lower_right = Position {
        x: upper_left.x + w,
        y: upper_left.y - h,
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
                    } else if p == view_center {
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
            .title(" Program ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL),
    )
    .style(Style::default().fg(Color::White).bg(Color::Black))
    .widths(&*widths)
    .column_spacing(0);

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
        .wrap(Wrap { trim: true });

    f.render_widget(program_grid, chunks[0]);
    f.render_widget(output, chunks[1]);
}
