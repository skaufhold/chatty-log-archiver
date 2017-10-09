
extern crate twitch_archiver;
extern crate dotenv;
extern crate diesel;
extern crate chrono;

use dotenv::dotenv;
use std::env;
use std::sync::{Once, ONCE_INIT};
use diesel::prelude::*;
use diesel::pg::PgConnection;
use chrono::prelude::*;
use twitch_archiver::schema::*;
use twitch_archiver::models::*;

use twitch_archiver::collector::{Collector, PgCollector, RawMessage};

static START: Once = ONCE_INIT;

fn connect() -> PgConnection {
    START.call_once(|| {
        dotenv().expect("Dotenv failed to load");
    });

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let conn = PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url));
    conn.begin_test_transaction().unwrap();
    conn
}

fn test_message() -> RawMessage {
    RawMessage {
        nick: "test".to_string(),
        channel: "testchannel".to_string(),
        message: "message here".to_string(),
        sent_at: Utc::now().naive_utc(),
        prime: false,
        moderator: false,
    }
}

#[test]
fn save_message() {
    let connection = connect();
    let raw_message = test_message();

    {
        let mut coll = PgCollector::new(&connection);
        coll.add_message(raw_message.clone()).unwrap();
    }

    let loaded_user: User = users::table
        .filter(users::name.eq(raw_message.nick))
        .first(&connection)
        .unwrap();

    let loaded_messages: Vec<Message> = Message::belonging_to(&loaded_user)
        .load(&connection)
        .unwrap();

    assert_eq!(loaded_messages[0].message, raw_message.message);
    assert_eq!(loaded_messages.len(), 1)
}
