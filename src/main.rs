extern crate clap;
extern crate fungoid;

use clap::{App, Arg};
use std::io;

fn main() {
    let matches = App::new("fungoid")
        .version("0.1.0")
        .author("Josh Karpel <josh.karpel@gmail.com>")
        .about("A Befunge interpreter written in Rust")
        .arg(
            Arg::with_name("FILE")
                .help("file to read program from")
                .required(true)
                .index(1),
        )
        .arg(Arg::with_name("time").long("time").help("enable timing"))
        .arg(
            Arg::with_name("show")
                .long("show")
                .help("show program before executing"),
        )
        .arg(
            Arg::with_name("step")
                .long("step")
                .help("step through program"),
        )
        .get_matches();

    let filename = matches.value_of("FILE").unwrap();

    let program = fungoid::Program::from_file(&filename);

    if matches.is_present("show") {
        program.show();
    }

    let input = &mut io::stdin();
    let output = &mut io::stdout();
    let error = &mut io::stderr();
    let program_state = fungoid::ProgramState::new(program, input, output, error);

    if matches.is_present("time") {
        fungoid::time(program_state);
    } else if matches.is_present("step") {
        fungoid::step(program_state);
    } else {
        fungoid::run_to_termination(program_state);
    }
}
