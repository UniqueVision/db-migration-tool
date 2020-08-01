#[macro_use]
extern crate clap;

mod error;

use clap::{Arg};
use error::SystemError;
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
    apply(client, sql).await?;
    Ok(())
}

fn add(
    patch_name: &str,
    sha1_code: &str,
) -> String {
    format!(r#"
    INSERT INTO public.migration_patches (
        patch_name
        ,sha1_code
        ,created_at
        ,updated_at
    ) VALUES (
        '{}'
        ,'{}'
        ,NOW()
        ,NOW()
    )
    ON CONFLICT (patch_name) DO UPDATE SET
        sha1_code = EXCLUDED.sha1_code
        ,updated_at = EXCLUDED.updated_at;

"#, patch_name, sha1_code)
}

async fn apply(
    client: &Client,
    sql: &str
) -> Result<u64, SystemError> {
    client
        .execute(sql, &[])
        .await.map_err(|err| err.into())
}

async fn is_exists(
    client: &Client,
    sha1_flag: bool,
    file_name: &str,
    next_sha1_code: &str,
) -> Result<bool, SystemError> {
    let sql = r#"
        SELECT
            t1.sha1_code
        FROM
            public.migration_patches AS t1
        WHERE
            t1.patch_name = $1
    "#;
    let rows = client
        .query(sql, &[&file_name])
        .await?;
    if rows.len() == 0 {
        return Ok(false);
    } else if !sha1_flag {
        return Ok(true);
    }
    let sha1_code: &str = rows[0].get(0);
    Ok(next_sha1_code == sha1_code)
}

async fn execute_dir(client: &Client, dir: &str, sha1_flag: bool, patch_file: &mut BufWriter<File>) -> Result<(), SystemError> {
    let entries = std::fs::read_dir(dir)?;
    let mut list = vec![];
    for entry in entries {
        let entry = entry?;
        let file_name = match entry.file_name().into_string() {
            Ok(str) => str,
            Err(_) => return Err(SystemError::Other("file_name convert error".to_owned()))
        };
        let mut file = BufReader::new(std::fs::File::open(entry.path())?);
        let mut s = String::new();
        let _ = file.read_to_string(&mut s)?;
        let sha = sha1::Sha1::from(&s).digest().to_string();
        list.push((file_name, sha, s));
    }
    list.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    for entry in list {
        if !is_exists(client, sha1_flag, &entry.0, &entry.1).await? {
            println!("{}", entry.0);
            patch_file.write(&entry.2.as_bytes())?;
            patch_file.write("\n\n".as_bytes())?;
            patch_file.write(&add(&entry.0, &entry.1).as_bytes())?;
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
