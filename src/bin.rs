use clap::{clap_app, crate_version, AppSettings};
use lasm::{assemble, target::C};
use std::{
    fs::{read_to_string, write},
    process::exit,
};

fn main() {
    let matches = clap_app!(lasm =>
        (version: crate_version!())
        (author: "Adam McDaniel <adam.mcdaniel17@gmail.com>")
        (about: "Compiles lasm assembly")
        (@arg input: +takes_value +required "Path to free file to compile")
        (@arg output: +takes_value "Path to output file")
    )
    .setting(AppSettings::ArgRequiredElseHelp)
    .get_matches();

    let output_file = match matches.value_of("output") {
        Some(file) => file,
        None => "out.c",
    };

    if let Some(file) = matches.value_of("input") {
        if let Ok(contents) = read_to_string(file) {
            let output_contents = match assemble(C, contents) {
                Ok(c) => c,
                Err(e) => {
                    println!("{}", e);
                    exit(1);
                }
            };

            if write(&output_file, &output_contents).is_ok() {
                println!("Successfully compiled program to {}", output_file);
            }
        }
    }
}
