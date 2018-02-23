use diesel::pg::Pg;
use diesel::row::Row;
use diesel::types::*;
use std::error::Error;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(i16)]
pub enum MessageFlag {
    Broadcaster = 1,
    Moderator = 2,
    Prime = 3,
    Subscriber = 4,
    Staff = 5
}

/*impl Into<i16> for MessageFlag {
    fn into(self) -> i16 {
        self as i16
    }
}*/

impl From<MessageFlag> for i16 {
    fn from(flag: MessageFlag) -> Self {
        flag as i16
    }
}

impl FromSqlRow<VarChar, Pg> for MessageFlag {
    fn build_from_row<R: Row<Pg>>(row: &mut R) -> Result<Self, Box<Error + Send + Sync>> {
        match i16::build_from_row(row)? {
            1 => Ok(MessageFlag::Broadcaster),
            2 => Ok(MessageFlag::Moderator),
            3 => Ok(MessageFlag::Prime),
            4 => Ok(MessageFlag::Subscriber),
            5 => Ok(MessageFlag::Staff),
            v => Err(format!("Unknown value {} for MessageFlag found", v).into()),
        }
    }
}
