use std::{
    error::Error,
    fmt,
    fmt::Display,
    io,
    io::{Read, Write},
    str::FromStr,
    string::String,
    time::Instant,
};

use clap::{
    Arg,
    ArgAction::{Set, SetTrue},
    ArgMatches, Command,
};
use fungoid::{examples::EXAMPLES, execution::ExecutionState, program::Program};
use humantime::format_duration;
use itertools::Itertools;
use separator::Separatable;

fn main() {
    if let Err(e) = cli() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

type GenericResult = Result<(), Box<dyn std::error::Error>>;

fn command() -> Command {
    Command::new("fungoid")
        .version("0.2.1")
        .author("Josh Karpel <josh.karpel@gmail.com>")
        .about("A Befunge interpreter written in Rust")
        .subcommand(
            Command::new("run")
                .about("Execute a program")
                .arg(
                    Arg::new("profile")
                        .long("profile")
                        .action(SetTrue)
                        .help("Enable profiling"),
                )
                .arg(
                    Arg::new("trace")
                        .long("trace")
                        .action(SetTrue)
                        .help("Trace program execution"),
                )
                .arg(
                    Arg::new("FILE")
                        .action(Set)
                        .required(true)
                        .help("The file to read the program from"),
                ),
        )
        .subcommand(
            Command::new("ide").about("Start a TUI IDE").arg(
                Arg::new("FILE")
                    .action(Set)
                    .required(true)
                    .help("The file to read the program from"),
            ),
        )
        .subcommand(
            Command::new("examples").about("Print the names of the bundled example programs."),
        )
        .arg_required_else_help(true)
}
fn cli() -> GenericResult {
    let matches = command().get_matches();

    if let Some(matches) = matches.subcommand_matches("ide") {
        ide(matches)?;
    } else if let Some(matches) = matches.subcommand_matches("run") {
        run_program(matches)?;
    } else if matches.subcommand_matches("examples").is_some() {
        println!("{}", EXAMPLES.keys().sorted().join("\n"))
    }

    Ok(())
}

#[derive(Debug)]
struct NoExampleFound {
    msg: String,
}

impl NoExampleFound {
    fn new(msg: String) -> NoExampleFound {
        NoExampleFound { msg }
    }
}

impl Display for NoExampleFound {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl Error for NoExampleFound {
    fn description(&self) -> &str {
        &self.msg
    }
}

fn load_program(matches: &ArgMatches) -> Result<Program, Box<dyn Error>> {
    let file = matches.get_one::<String>("FILE").unwrap();
    if file.starts_with("example:") || file.starts_with("examples:") {
        let (_, e) = file.split_once(':').unwrap();
        if let Some(p) = EXAMPLES.get(e) {
            Ok(Program::from_str(p)?)
        } else {
            Err(Box::new(NoExampleFound::new(format!(
                "No example named '{}'.\nExamples: {:?}",
                e,
                EXAMPLES.keys()
            ))))
        }
    } else {
        Ok(Program::from_file(file)?)
    }
}

fn ide(matches: &ArgMatches) -> GenericResult {
    let program = load_program(matches)?;

    fungoid::ide::ide(program)?;

    Ok(())
}

fn run_program(matches: &ArgMatches) -> GenericResult {
    let program = load_program(matches)?;

    let input = &mut io::stdin();
    let output = &mut io::stdout();
    let program_state =
        fungoid::execution::ExecutionState::new(program, matches.get_flag("trace"), input, output);

    run(program_state, matches.get_flag("profile"))?;

    Ok(())
}

pub fn run<R: Read, O: Write>(
    mut program_state: ExecutionState<R, O>,
    profile: bool,
) -> GenericResult {
    let start = Instant::now();
    program_state.run()?;
    let duration = start.elapsed();

    let num_seconds = 1.0e-9 * (duration.as_nanos() as f64);

    if profile {
        eprintln!(
            "Executed {} instructions in {} ({} instructions/second)",
            program_state.instruction_count,
            format_duration(duration),
            ((program_state.instruction_count as f64 / num_seconds) as u64).separated_string()
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::command;

    #[test]
    fn verify_command() {
        command().debug_assert();
    }
}
