use std::collections::HashMap;

pub const DNA: &str = include_str!("examples/dna.bf");
pub const ERATOSTHENES: &str = include_str!("examples/eratosthenes.bf");
pub const FACTORIAL: &str = include_str!("examples/factorial.bf");
pub const HELLO_WORLD: &str = include_str!("examples/hello_world.bf");
pub const INPUT: &str = include_str!("examples/input.bf");
pub const QUINE: &str = include_str!("examples/quine.bf");
pub const RNG: &str = include_str!("examples/rng.bf");

lazy_static! {
    pub static ref EXAMPLES: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("dna", DNA);
        m.insert("eratosthenes", ERATOSTHENES);
        m.insert("factorial", FACTORIAL);
        m.insert("hello_world", HELLO_WORLD);
        m.insert("input", INPUT);
        m.insert("quine", QUINE);
        m.insert("rng", RNG);
        m
    };
}
