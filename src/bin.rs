extern crate twitch_archiver;
extern crate diesel;
extern crate dotenv;
extern crate error_chain;

use twitch_archiver::parser::{LogParser, ChattyParser};
use twitch_archiver::collector::PgCollector;
use std::fs::File;
use std::env;
use std::io::{Write, BufReader};
use std::path::Path;
use diesel::Connection;
use diesel::pg::PgConnection;
use error_chain::ChainedError;

fn connect() -> PgConnection {
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

fn main() {
    ::dotenv::dotenv().ok();
    let args: Vec<String> = env::args().collect();
    let path = Path::new(args.get(1)
        .expect("Path parameter required"));

    if !path.exists() { panic!("File does not exist") };
    if !path.is_file() { panic!("Path is not a file") };

    println!("Opening log file {}", args[1]);
    let file = File::open(path)
        .expect("Couldn't open file");

    let connection = connect();
    let mut collector = PgCollector::new(&connection);
    match ChattyParser::parse(&mut collector, BufReader::new(file)) {
        Err(err) => { writeln!(&mut ::std::io::stderr(), "{}", err.display_chain()).unwrap(); },
        _ => {}
    }
}
