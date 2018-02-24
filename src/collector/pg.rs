use collector::*;
use diesel;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use errors::Result;
use models::{Channel, NewChannel, NewMessage, NewUser, User};
use schema::{channels, messages, users};
use std::collections::HashMap;

pub struct PgCollector<'a> {
    batch_size: usize,
    message_batch: Vec<NewMessage>,
    channel_map: HashMap<String, i32>,
    user_map: HashMap<String, i32>,
    connection: &'a PgConnection
}

impl <'a> Collector for PgCollector<'a> {
    fn add_message(&mut self, raw_message: RawMessage) -> Result<()> {
        let new_user_id: i32 = self.find_or_create_user_id(raw_message.nick.as_ref())?;
        let new_channel_id: i32 = self.find_or_create_channel_id(raw_message.channel.as_ref())?;

        let new_message = NewMessage {
            user_id: new_user_id,
            channel_id: new_channel_id,
            message: raw_message.message,
            sent_at: raw_message.sent_at,
            flags: raw_message.flags.into_iter().map(|f| f.into()).collect()
        };

        self.message_batch.push(new_message);

        if self.message_batch.len() >= self.batch_size {
            self.commit()?;
        }

        Ok(())
    }

    fn commit(&mut self) -> Result<()> {
        diesel::insert(&self.message_batch).into(messages::table)
            .execute(self.connection)?;
        self.message_batch.clear();
        Ok(())
    }
}

impl <'a> PgCollector<'a> {
    pub fn new(connection: &PgConnection) -> PgCollector {
        PgCollector {
            batch_size: DEFAULT_BATCH_SIZE,
            message_batch: Vec::new(),
            channel_map: HashMap::new(),
            user_map: HashMap::new(),
            connection,
        }
    }

    fn find_user_by_nick(&self, nick: &str) -> Option<User> {
        use self::users::dsl::*;
        let user = users.filter(name.eq(nick))
            .first(self.connection);

        user.ok()
    }

    fn find_channel_by_name(&self, channel: &str) -> Option<Channel> {
        use self::channels::dsl::*;
        let channel = channels.filter(name.eq(channel))
            .first(self.connection);

        channel.ok()
    }

    fn find_or_create_channel_id(&self, name: &str) -> Result<i32> {
        if let Some(id) = self.channel_map.get(name) {
            Ok(*id)
        } else if let Some(channel) = self.find_channel_by_name(name) {
            Ok(channel.id)
        } else {
            let new_channel = NewChannel { name };
            diesel::insert(&new_channel).into(channels::table)
                .get_result(self.connection)
                .map(|c: Channel| c.id)
                .map_err(|e| e.into())
        }
    }

    fn find_or_create_user_id(&self, name: &str) -> Result<i32> {
        if let Some(id) = self.user_map.get(name) {
            Ok(*id)
        } else if let Some(user) = self.find_user_by_nick(name) {
            Ok(user.id)
        } else {
            let new_user = NewUser {
                name,
            };
            diesel::insert(&new_user).into(users::table)
                .get_result(self.connection)
                .map(|u: User| u.id)
                .map_err(|e| e.into())
        }
    }
}

impl <'a> Drop for PgCollector<'a> {
    fn drop(&mut self) {
        self.commit().unwrap();
    }
}
