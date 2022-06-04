use std::{io::prelude::*, time::Instant};

use humantime::format_duration;
use separator::Separatable;

use execution::ExecutionState;
use program::{Position, Program};

pub mod execution;
pub mod ide;
pub mod program;

pub fn run_to_termination<O: Write>(program_state: ExecutionState<O>) -> u64 {
    program_state.run().instruction_count
}

pub fn time<O: Write>(program_state: ExecutionState<O>) {
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

#[cfg(test)]
mod tests {
    use std::io;
    use std::str::FromStr;

    use crate::execution::ExecutionState;
    use crate::program::Program;

    const HELLO_WORLD: &str = r#"64+"!dlroW ,olleH">:#,_@"#;

    #[test]
    fn hello_world() -> Result<(), io::Error> {
        let program = Program::from_str(HELLO_WORLD)?;
        let mut output = Vec::new();
        ExecutionState::new(program, false, &mut io::stdin(), &mut output).run();
        println!("{:?}", output);
        assert_eq!("Hello, World!\n", String::from_utf8(output).unwrap());

        Ok(())
    }

    const SIEVE_OF_ERATOSTHENES: &str = r#"2>:3g" "-!v\  g30          <
 |!`"O":+1_:.:03p>03g+:"O"`|
 @               ^  p3\" ":<
2 234567890123456789012345678901234567890123456789012345678901234567890123456789
"#;

    #[test]
    fn sieve_of_eratosthenes() -> Result<(), io::Error> {
        let program = Program::from_str(SIEVE_OF_ERATOSTHENES)?;
        let mut output = Vec::new();
        ExecutionState::new(program, false, &mut io::stdin(), &mut output).run();
        println!("{:?}", output);
        assert_eq!(
            "2357111317192329313741434753596167717379",
            String::from_utf8(output).unwrap()
        );

        Ok(())
    }

    const QUINE: &str = r#"01->1# +# :# 0# g# ,# :# 5# 8# *# 4# +# -# _@"#;

    #[test]
    fn quine() -> Result<(), io::Error> {
        let program = Program::from_str(QUINE)?;
        let mut output = Vec::new();
        ExecutionState::new(program, false, &mut io::stdin(), &mut output).run();
        println!("{:?}", output);
        assert_eq!(
            "01->1# +# :# 0# g# ,# :# 5# 8# *# 4# +# -# _@",
            String::from_utf8(output).unwrap()
        );

        Ok(())
    }
}
