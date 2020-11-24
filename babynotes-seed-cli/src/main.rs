extern crate babynotes_mutator;
extern crate bincode;
extern crate clap;
extern crate fuzzer;

use std::fs::File;
use std::path::{Path, PathBuf};

use babynotes_mutator::Input;
use fuzzer::Command;

const CMD_EXIT: i32 = 7;

fn main() {
    let matches = clap::App::new("babynotes-seed-gen")
        .version("0.1")
        .author("Sirui Mu <msrlancern@126.com>")
        .about("Command line utility to manipulate seed files for fuzzing babynotes")
        .subcommand(
            clap::SubCommand::with_name("gen")
                .about("Generate seed file")
                .arg(
                    clap::Arg::with_name("output")
                        .short("o")
                        .long("output")
                        .value_name("FILE")
                        .required(true)
                        .help("Set the path to the output seed file")
                        .takes_value(true),
                ),
        )
        .subcommand(
            clap::SubCommand::with_name("syn")
                .about("Synthesis a seed")
                .arg(
                    clap::Arg::with_name("file")
                        .required(true)
                        .value_name("FILE")
                        .help("Path to the seed file"),
                )
                .arg(
                    clap::Arg::with_name("output")
                        .short("o")
                        .long("output")
                        .value_name("OUTPUT")
                        .required(true)
                        .help("Path to the output file")
                        .takes_value(true),
                ),
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("gen") {
        let path = PathBuf::from(String::from(matches.value_of("output").unwrap()));
        generate(&path);
    } else if let Some(matches) = matches.subcommand_matches("syn") {
        let input_path = PathBuf::from(String::from(matches.value_of("file").unwrap()));
        let output_path = PathBuf::from(String::from(matches.value_of("output").unwrap()));
        synthesis(&input_path, &output_path);
    } else {
        eprintln!("Error: no commands given");
    }
}

fn generate(path: &Path) {
    if path.exists() {
        eprintln!("Error: cannot overwrite existing seed file.");
        return;
    }

    let mut file = match File::create(path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Error: cannot create file: {}", e);
            return;
        }
    };

    let mut input = Input::new();
    input.commands.commands.push(Command {
        id: CMD_EXIT,
        data: vec![],
    });

    match bincode::serialize_into(&mut file, &input) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Error: cannot write seed input: {}", e);
            return;
        }
    };
}

fn synthesis(input_path: &Path, output_path: &Path)  {
    if !input_path.exists() {
        eprintln!("Error: no such seed file: {}", input_path.display());
        return;
    }

    if output_path.exists() {
        eprintln!("Error: cannot overwrite existing file");
        return;
    }

    let input = {
        let file = match File::open(input_path) {
            Ok(file) => file,
            Err(e) => {
                eprintln!("Error: cannot open input file: {}", e);
                return;
            },
        };
        match bincode::deserialize_from::<File, Input>(file) {
            Ok(input) => input,
            Err(e) => {
                eprintln!("Error: cannot deserialize seed input: {}", e);
                return;
            },
        }
    };

    let file = match File::create(output_path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Error: cannot create output file: {}", e);
            return;
        },
    };

    match bincode::serialize_into(file, &input) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Error: cannot write synthesised input: {}", e);
            return;
        }
    };
}
