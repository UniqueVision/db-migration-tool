#[macro_use]
extern crate clap;

mod error;
mod file_entry;

use clap::{Arg};
use error::SystemError;
use file_entry::FileEntry;
use tokio_postgres::{Client, NoTls};
use std::{
    fs::File,
    io::{BufReader, BufWriter, Write, Read},
};

// cargo run -- -u postgres://user:pass@localhost:5432/test -d ./patches -i -r

static PATH_FILE: &str = "/tmp/patch.sql";

async fn create_table(
    client: &Client,
) -> Result<(), SystemError> {
    let sql = r#"
        CREATE TABLE IF NOT EXISTS public.migration_patches (
            patch_name TEXT NOT NULL
            ,sha1_code TEXT NOT NULL
            ,created_at TIMESTAMPTZ NOT NULL
            ,updated_at TIMESTAMPTZ NOT NULL
            ,PRIMARY KEY (patch_name)
        );
    "#;
    client
        .execute(sql, &[])
        .await?;
    Ok(())
}

async fn execute_dir(client: &Client, dir: &str, sha1_flag: bool, patch_file: &mut BufWriter<File>) -> Result<(), SystemError> {
    let entries = std::fs::read_dir(dir)?;
    let mut list = vec![];
    for entry in entries {
        let entry = entry?;
        list.push(FileEntry::new(&entry)?);
    }
    list.sort();
    for entry in list {
        if !entry.is_exists(client, sha1_flag).await? {
            println!("{}", entry.file_name);
            entry.write(patch_file)?;
        }
    }
    Ok(())
}

async fn apply_all(client: &Client) -> Result<(), SystemError> {
    let mut file = BufReader::new(std::fs::File::open(PATH_FILE)?);
    let mut s = String::new();
    let _ = file.read_to_string(&mut s)?;
    client
        .batch_execute(&s)
        .await?;
    Ok(())
}

async fn execute(
    url: &str,
    dirs: Vec<&str>,
    sha1_flag: bool,
    init_flag: bool,
    dry_flag: bool,
) -> Result<(), SystemError> {
    let (client, connection) = tokio_postgres::connect(url, NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    if init_flag {
        create_table(&client).await?;
    }

    {
        let mut file = BufWriter::new(std::fs::File::create(PATH_FILE).unwrap());

        for dir in dirs {
            execute_dir(&client, dir, sha1_flag, &mut file).await?;
        }

        file.flush()?;
    }
    if !dry_flag {
        apply_all(&client).await?;
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    let matches = app_from_crate!()
        .arg(
            Arg::with_name("url")
            .short("u")
            .long("url")
            .help("postgresql url")
            .required(true)
            .takes_value(true)
        )
        .arg(Arg::with_name("dirs")
            .help("Target Dir")
            .short("d")
            .long("dir")
            .multiple(true)
            .takes_value(true)
        )
        .arg(Arg::with_name("sha1_flag")
            .help("Use sha1")
            .short("s")
            .long("sha1")
        )
        .arg(Arg::with_name("init_flag")
            .help("Create Table")
            .short("i")
            .long("init")
        )
        .arg(Arg::with_name("dry_flag")
            .help("Dry Run")
            .short("r")
            .long("dry_run")
        )
        .get_matches();

    let dirs: Vec<_> = match matches.values_of("dirs") {
        Some(values) => values.collect(),
        None => vec![]
    };
    let sha1_flag = matches.is_present("sha1_flag");
    let init_flag = matches.is_present("init_flag");
    let dry_flag = matches.is_present("dry_flag");
    let url = matches.value_of("url").unwrap();
    match execute(url, dirs, sha1_flag, init_flag, dry_flag).await {
        Ok(_) => {},
        Err(err) => println!("{:?}", err),
    }
}
