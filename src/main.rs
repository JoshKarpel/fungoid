extern crate clap;
extern crate fungoid;

use clap::{App, Arg};

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
        .get_matches();

    let filename = matches.value_of("FILE").unwrap();

    let program = fungoid::Program::from_file(&filename);

    println!("{}", vec!["-"; 80].join(""));
    println!("{}", program);
    println!("{}", vec!["-"; 80].join(""));

    println!("OUTPUT");
    if matches.is_present("time") {
        fungoid::time(program);
    } else {
        fungoid::run(program);
    }
}
