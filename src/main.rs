use std::{
    error::Error, ffi::OsString, fmt, fmt::Display, io, str::FromStr, string::String, time::Instant,
};

use clap::{Args, Parser, Subcommand};
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

type GenericResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Parser)]
#[command(name = "fungoid", author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Run a program
    #[command(arg_required_else_help = true)]
    Run {
        /// The path to the file to read the program from
        file: OsString,
        /// Enable execution tracing
        #[arg(long)]
        trace: bool,
        /// Enable profiling
        #[arg(long)]
        profile: bool,
    },
    /// Start the TUI IDE
    #[command(arg_required_else_help = true)]
    Ide {
        /// The path to the file to open
        file: OsString,
    },
    /// Interact with the bundled example programs.
    #[command(arg_required_else_help = true)]
    Examples(ExamplesArgs),
}

#[derive(Debug, Args)]
struct ExamplesArgs {
    #[command(subcommand)]
    command: ExamplesCommands,
}

#[derive(Debug, Subcommand)]
enum ExamplesCommands {
    /// Print the available bundled example programs
    #[command(arg_required_else_help = true)]
    List,
    /// Print one of the example programs to stdout
    #[command(arg_required_else_help = true)]
    Print { example: String },
    /// Run one of the example programs
    #[command(arg_required_else_help = true)]
    Run {
        /// The name of the example to run
        example: String,
        /// Enable execution tracing
        #[arg(long)]
        trace: bool,
        /// Enable profiling
        #[arg(long)]
        profile: bool,
    },
}

fn cli() -> GenericResult<()> {
    match Cli::parse().command {
        Commands::Run {
            file,
            trace,
            profile,
        } => {
            let program = Program::from_file(&file)?;

            run_program(program, trace, profile)?;

            Ok(())
        }

        Commands::Ide { file } => {
            let program = Program::from_file(&file)?;

            fungoid::ide::ide(program)?;

            Ok(())
        }

        Commands::Examples(ExamplesArgs {
            command: ExamplesCommands::List,
        }) => {
            println!("{}", EXAMPLES.keys().sorted().join("\n"));

            Ok(())
        }

        Commands::Examples(ExamplesArgs {
            command: ExamplesCommands::Print { example },
        }) => {
            let program = get_example(example.as_str())?;
            println!("{}", program);

            Ok(())
        }

        Commands::Examples(ExamplesArgs {
            command:
                ExamplesCommands::Run {
                    example,
                    trace,
                    profile,
                },
        }) => {
            let program = Program::from_str(get_example(example.as_str())?).unwrap();

            run_program(program, trace, profile)?;

            Ok(())
        }
    }
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

fn get_example(example: &str) -> GenericResult<&str> {
    if let Some(program) = EXAMPLES.get(example) {
        Ok(program)
    } else {
        Err(Box::new(NoExampleFound::new(format!(
            "No example named '{}'.\nExamples:\n{}",
            example,
            EXAMPLES.keys().sorted().join("\n")
        ))))
    }
}

fn run_program(program: Program, trace: bool, profile: bool) -> GenericResult<()> {
    let input = &mut io::stdin();
    let output = &mut io::stdout();
    let mut program_state = ExecutionState::new(program, trace, input, output);

    let start = Instant::now();
    program_state.run()?;
    let duration = start.elapsed();

    if profile {
        eprintln!(
            "Executed {} instructions in {} ({} instructions/second)",
            program_state.instruction_count,
            format_duration(duration),
            ((program_state.instruction_count as f64 / duration.as_secs_f64()) as u64)
                .separated_string()
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use crate::Cli;

    #[test]
    fn verify_command() {
        Cli::command().debug_assert()
    }
}
