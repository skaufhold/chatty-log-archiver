extern crate diesel;
extern crate error_chain;
#[macro_use] extern crate quicli;
extern crate twitch_archiver;
extern crate core;

use diesel::Connection;
use diesel::pg::PgConnection;
use error_chain::ChainedError;
use quicli::prelude::*;
use std::fs::File;
use std::io::{BufReader};
use std::path::PathBuf;
use twitch_archiver::collector::PgCollector;
use twitch_archiver::parser::{ChattyParser, LogParser};
use core::str::FromStr;


/// Parse chatty log files and save the data in other formats or databases
#[derive(Debug, StructOpt)]
struct Cli {
    /// Output/Exporting format (pg, csv)
    #[structopt(long = "output", short = "o")]
    format: Output,
    /// Database URL if saving in a database
    #[structopt(long = "database", short = "db")]
    db_url: Option<String>,
    /// The log file to read
    #[structopt(parse(from_os_str))]
    file: PathBuf,
}

#[derive(Debug)]
enum Output {
    Pg,
    Csv,
}

impl FromStr for Output {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "pg" | "postgres" => Ok(Output::Pg),
            "csv" => Ok(Output::Csv),
            _ => bail!("Invalid output type")
        }
    }
}

main!(|args: Cli| {
    if !args.file.exists() { bail!("File does not exist") };
    if !args.file.is_file() { bail!("Path is not a file") };

    println!("Opening log file {:?}", args.file);
    let file = File::open(args.file)?;

    match args.format {
        Output::Pg => {
            let connection = PgConnection::establish(
                args.db_url
                    .ok_or_else(|| format_err!("Database URL missing"))?
                    .as_ref()
            )?;
            let mut collector = PgCollector::new(&connection);
            if let Err(err) = ChattyParser::parse(&mut collector, BufReader::new(file)) {
                error!("{}", err.display_chain());
            }
        },
        Output::Csv => {}
    }
});
