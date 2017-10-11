#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageFlag {
    Broadcaster,
    Moderator,
    Prime,
    Subscriber,
    Staff
}

pub(crate) mod sql {
    use diesel::pg::{Pg, PgMetadataLookup};
    use diesel::row::Row;
    use diesel::types::*;
    use std::error::Error;
    use std::io::Write;

    // This struct represents the SQL type in PG. It should have the same name as in PG.
    pub struct MessageFlag;

    impl HasSqlType<MessageFlag> for Pg {
        fn metadata(lookup: &PgMetadataLookup) -> Self::TypeMetadata {
            lookup.lookup_type("message_flag")
        }
    }

    impl NotNull for MessageFlag {}

    impl FromSql<MessageFlag, Pg> for super::MessageFlag {
        fn from_sql(bytes: Option<&[u8]>) -> Result<Self, Box<Error + Send + Sync>> {
            match bytes {
                Some(b"moderator") => Ok(super::MessageFlag::Moderator),
                Some(b"prime") => Ok(super::MessageFlag::Prime),
                Some(b"broadcaster") => Ok(super::MessageFlag::Broadcaster),
                Some(b"subscriber") => Ok(super::MessageFlag::Subscriber),
                Some(b"staff") => Ok(super::MessageFlag::Staff),
                Some(_) => Err("Invalid message flag variant".into()),
                None => Err("Unexpected null for non-null column".into()),
            }
        }
    }

    impl ToSql<MessageFlag, Pg> for super::MessageFlag {
        fn to_sql<W: Write>(&self, out: &mut ToSqlOutput<W, Pg>) -> Result<IsNull, Box<Error + Send + Sync>> {
            let bytes: &'static [u8] = match *self {
                super::MessageFlag::Moderator => b"moderator",
                super::MessageFlag::Prime => b"prime",
                super::MessageFlag::Broadcaster => b"broadcaster",
                super::MessageFlag::Subscriber => b"subscriber",
                super::MessageFlag::Staff => b"staff",
            };
            out.write_all(bytes)
                .map(|_| IsNull::No)
                .map_err(|e| e.into())
        }
    }

    // You shouldn't need to implement this one manually, but I need to fix something upstream
    impl FromSqlRow<MessageFlag, Pg> for super::MessageFlag {
        fn build_from_row<R: Row<Pg>>(row: &mut R) -> Result<Self, Box<Error + Send + Sync>> {
            FromSql::<MessageFlag, Pg>::from_sql(row.take())
        }
    }
}
