#[macro_use]
extern crate clap;
use clap::{Arg};

fn main() {
    let matches = app_from_crate!()
        .arg(
            Arg::with_name("url")
            .short("u")
            .long("url")
            .help("postgresql url")
            .required(true)
            .takes_value(true)
        )
        .arg(Arg::with_name("patch_dirs")
            .help("Patch Dir")
            .short("p")
            .long("patch_dir")
            .multiple(true)
            .takes_value(true)
        )
        .arg(Arg::with_name("stored_dirs")
            .help("Stored Dir")
            .short("s")
            .long("stored_dir")
            .multiple(true)
            .takes_value(true)
        ).get_matches();

    let patch_dirs: Vec<_> = match matches.values_of("patch_dirs") {
        Some(values) => values.collect(),
        None => vec![]
    };
    let stored_dirs: Vec<_> = match matches.values_of("stored_dirs") {
        Some(values) => values.collect(),
        None => vec![]
    };
    println!("{:?}", patch_dirs);
}
