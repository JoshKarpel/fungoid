extern crate fungoid;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];

    let program = fungoid::Program::from_file(&filename);

    println!("{}", vec!["-"; 80].join(""));
    println!("{}", program);
    println!("{}", vec!["-"; 80].join(""));

    println!("OUTPUT");
//    run(program);
    fungoid::benchmark(program);
}
