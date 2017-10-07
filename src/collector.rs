
use models::{Message, NewMessage, NewUser, User, Channel, NewChannel};
use std::collections::HashMap;
use chrono::NaiveDateTime;
use schema::{users, channels, messages};
use diesel;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use errors::Result;

static DEFAULT_BATCH_SIZE: usize = 300;

pub struct Collector<'a> {
    batch_size: usize,
    message_batch: Vec<NewMessage>,
    channel_map: HashMap<String, i32>,
    user_map: HashMap<String, i32>,
    connection: &'a PgConnection
}

impl <'a> Collector<'a> {
    pub fn new(connection: &PgConnection) -> Collector {
        Collector {
            batch_size: DEFAULT_BATCH_SIZE,
            message_batch: Vec::new(),
            channel_map: HashMap::new(),
            user_map: HashMap::new(),
            connection,
        }
    }

    pub fn add_message(&mut self, raw_message: RawMessage) -> Result<()> {
        let new_user_id: i32 = match self.user_map.get(&raw_message.nick) {
            Some(id) => *id,
            None => {
                self.find_user_by_nick(raw_message.nick.as_ref())
                    .or_else(|| {
                        let new_user = NewUser {
                            name: raw_message.nick.as_ref(),
                        };
                        let inserted_user: User = diesel::insert(&new_user).into(users::table)
                            .get_result(self.connection)
                            .expect("Error saving new user");
                        Some(inserted_user)
                    })
                    .unwrap().id
            }
        };

        let new_channel_id: i32 = match self.channel_map.get(&raw_message.channel) {
            Some(id) => *id,
            None => {
                self.find_channel_by_name(raw_message.channel.as_ref())
                    .or_else(|| {
                        let new_channel = NewChannel {
                            name: raw_message.channel.as_ref()
                        };
                        let inserted_channel: Channel = diesel::insert(&new_channel).into(channels::table)
                            .get_result(self.connection)
                            .expect("Error saving new channel");
                        Some(inserted_channel)
                    })
                    .unwrap().id
            }
        };

        let new_message = NewMessage {
            user_id: new_user_id,
            channel_id: new_channel_id,
            message: raw_message.message,
            sent_at: raw_message.sent_at,
            prime: raw_message.prime,
            moderator: raw_message.moderator,
        };

        self.message_batch.push(new_message);

        if self.message_batch.len() > self.batch_size {
            self.commit().unwrap();
        }

        Ok(())
    }

    pub fn commit(&mut self) -> Result<()> {
        let inserted_message = diesel::insert(&self.message_batch).into(messages::table)
            .execute(self.connection)?;
        self.message_batch.clear();
        Ok(())
    }

    fn find_user_by_nick(&self, nick: &str) -> Option<User> {
        use self::users::dsl::*;
        let user = users.filter(name.eq(nick))
            .first(self.connection);

        user.ok()
    }

    fn find_channel_by_name(&self, name: &str) -> Option<Channel> {
        use self::channels::dsl::*;
        let channel = channels.filter(name.eq(name))
            .first(self.connection);

        channel.ok()
    }
}

impl <'a> Drop for Collector<'a> {
    fn drop(&mut self) {
        self.commit().unwrap();
    }
}

#[derive(Clone)]
pub struct RawMessage {
    pub nick: String,
    pub channel: String,
    pub message: String,
    pub sent_at: NaiveDateTime,
    pub prime: bool,
    pub moderator: bool
}
