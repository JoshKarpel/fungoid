extern crate clap;
extern crate fungoid;

use clap::{App, Arg, ArgMatches, SubCommand};
use std::io;
use std::time::Duration;

fn main() {
    if let Err(e) = _main() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

type MainError = Result<(), Box<dyn std::error::Error>>;

fn _main() -> MainError {
    let matches = App::new("fungoid")
        .version("0.1.0")
        .author("Josh Karpel <josh.karpel@gmail.com>")
        .about("A Befunge interpreter written in Rust")
        .subcommand(
            SubCommand::with_name("run")
                .arg(Arg::with_name("time").long("time").help("enable timing"))
                .arg(
                    Arg::with_name("show")
                        .long("show")
                        .help("show program before executing"),
                )
                .arg(
                    Arg::with_name("trace")
                        .long("trace")
                        .help("trace program execution"),
                )
                .arg(
                    Arg::with_name("FILE")
                        .help("file to read program from")
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("step")
                .arg(
                    Arg::with_name("rate")
                        .long("rate")
                        .takes_value(true)
                        .default_value("10")
                        .help("maximum instructions per second"),
                )
                .arg(
                    Arg::with_name("FILE")
                        .help("file to read program from")
                        .required(true),
                ),
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("step") {
        step(matches)?;
    } else if let Some(matches) = matches.subcommand_matches("run") {
        run(matches)?;
    }

    Ok(())
}

fn step(matches: &ArgMatches) -> MainError {
    let program = fungoid::Program::from_file(&matches.value_of("FILE").unwrap())?;

    let max_ips: u32 = matches.value_of("rate").unwrap().parse()?;
    let dur = Duration::from_secs_f64(match max_ips {
        0 => 0.0,
        _ => 1.0 / (max_ips as f64),
    });

    fungoid::step(program, dur)?;

    Ok(())
}

fn run(matches: &ArgMatches) -> MainError {
    let program = fungoid::Program::from_file(&matches.value_of("FILE").unwrap())?;

    if matches.is_present("show") {
        program.show();
    }

    let input = &mut io::stdin();
    let output = &mut io::stdout();
    let program_state =
        fungoid::ProgramState::new(program, matches.is_present("trace"), input, output);

    if matches.is_present("time") {
        fungoid::time(program_state);
    } else {
        fungoid::run_to_termination(program_state);
    }

    Ok(())
}
