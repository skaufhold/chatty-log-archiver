table! {
    channels (id) {
        id -> Int4,
        name -> Varchar,
    }
}

table! {
    messages (id) {
        id -> Int4,
        user_id -> Int4,
        channel_id -> Int4,
        message -> Text,
        flags -> ::models::types::sql::MessageFlag,
        sent_at -> Timestamp,
        prime -> Bool,
        moderator -> Bool,
    }
}

table! {
    users (id) {
        id -> Int4,
        name -> Varchar,
    }
}

joinable!(messages -> users (user_id));
joinable!(messages -> channels (channel_id));
