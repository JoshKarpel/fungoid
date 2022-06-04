use std::io;
use std::time::{Duration, Instant};

use crossterm::{
    event,
    event::{poll, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap},
    Frame, Terminal,
};

use crate::execution::ExecutionState;
use crate::program::Program;

pub fn step(program: Program) -> crossterm::Result<()> {
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
    loop {
        terminal.draw(|f| ui(f, &program_state))?;

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
                    KeyCode::Char('+') => max_ips = (max_ips + 1).max(1),
                    KeyCode::Char('-') => max_ips = (max_ips - 1).max(1),
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

fn ui<B: Backend>(f: &mut Frame<B>, program_state: &ExecutionState<Vec<u8>>) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
        .split(f.size());

    let program_grid = Table::new(program_state.program.0.iter().enumerate().map(|(y, row)| {
        Row::new(row.iter().enumerate().map(|(x, c)| {
            Cell::from(c.to_string()).style(
                if program_state.pointer.position.x == x && program_state.pointer.position.y == y {
                    if program_state.terminated {
                        Style::default().bg(Color::Red)
                    } else {
                        Style::default().bg(Color::Green)
                    }
                } else {
                    Style::default().fg(Color::White)
                },
            )
        }))
    }))
    .block(
        Block::default()
            .title(" Program ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL),
    )
    .style(Style::default().fg(Color::White).bg(Color::Black))
    .widths(&[Constraint::Length(1); 30])
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
