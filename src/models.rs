
use chrono::naive::NaiveDateTime;
use chrono::DateTime;
use schema::*;

#[derive(Debug, Identifiable, Queryable)]
pub struct User {
    pub id: i32,
    pub name: String
}

#[derive(Debug, Insertable)]
#[table_name="users"]
pub struct NewUser<'a> {
    pub name: &'a str
}

#[derive(Debug, Identifiable, Queryable)]
pub struct Channel {
    pub id: i32,
    pub name: String
}

#[derive(Debug, Insertable)]
#[table_name="channels"]
pub struct NewChannel<'a> {
    pub name: &'a str
}

#[derive(Debug, Identifiable, Queryable, Associations, PartialEq)]
#[belongs_to(User)]
#[belongs_to(Channel)]
pub struct Message {
    pub id: i32,
    pub user_id: i32,
    pub channel_id: i32,
    pub message: String,
    pub sent_at: NaiveDateTime,
    pub prime: bool,
    pub moderator: bool
}

#[derive(Debug, Insertable)]
#[table_name="messages"]
pub struct NewMessage {
    pub user_id: i32,
    pub channel_id: i32,
    pub message: String,
    pub sent_at: NaiveDateTime,
    pub prime: bool,
    pub moderator: bool
}
