extern crate bincode;
extern crate clap;
extern crate fuzzer;

use std::fs::File;
use std::path::PathBuf;

use fuzzer::{Input, Command};

const CMD_EXIT: i32 = 7;

fn main() {
    let matches = clap::App::new("babynotes-seed-gen")
        .version("0.1")
        .author("Sirui Mu <msrlancern@126.com>")
        .about("Generate seed file for fuzzing babynotes")
        .arg(
            clap::Arg::with_name("output")
                .short("o")
                .long("output")
                .value_name("FILE")
                .required(true)
                .help("Set the path to the output seed file")
                .takes_value(true),
        )
        .get_matches();

    let output_path = PathBuf::from(String::from(matches.value_of("output").unwrap()));
    if output_path.exists() {
        eprintln!("Cannot overwrite existing seed file.");
        return;
    }

    let mut file = match File::create(&output_path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Error: cannot create file: {}", e);
            return;
        },
    };

    let mut input = Input::new();
    input.commands.push(Command {
        id: CMD_EXIT,
        data: vec![],
    });

    match bincode::serialize_into(&mut file, &input) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Error: cannot write seed input: {}", e);
            return;
        }
    }
}
