extern crate clap;
extern crate fungoid;

use std::io;

use clap::{Arg, ArgMatches, Command};

fn main() {
    if let Err(e) = _main() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

type MainError = Result<(), Box<dyn std::error::Error>>;

fn _main() -> MainError {
    let matches = Command::new("fungoid")
        .version("0.2.0")
        .author("Josh Karpel <josh.karpel@gmail.com>")
        .about("A Befunge interpreter written in Rust")
        .subcommand(
            Command::new("run")
                .arg(Arg::new("time").long("time").help("enable timing"))
                .arg(
                    Arg::new("trace")
                        .long("trace")
                        .help("trace program execution"),
                )
                .arg(
                    Arg::new("FILE")
                        .help("file to read program from")
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("ide").arg(
                Arg::new("FILE")
                    .help("file to read program from")
                    .required(true),
            ),
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("ide") {
        ide(matches)?;
    } else if let Some(matches) = matches.subcommand_matches("run") {
        run(matches)?;
    }

    Ok(())
}

fn ide(matches: &ArgMatches) -> MainError {
    let program = fungoid::program::Program::from_file(matches.value_of("FILE").unwrap())?;

    fungoid::ide::ide(program)?;

    Ok(())
}

fn run(matches: &ArgMatches) -> MainError {
    let program = fungoid::program::Program::from_file(matches.value_of("FILE").unwrap())?;

    let input = &mut io::stdin();
    let output = &mut io::stdout();
    let program_state = fungoid::execution::ExecutionState::new(
        program,
        matches.is_present("trace"),
        input,
        output,
    );

    if matches.is_present("time") {
        fungoid::time(program_state);
    } else {
        fungoid::run_to_termination(program_state);
    }

    Ok(())
}
