table! {
    note (id) {
        id -> Uuid,
        user_id -> Uuid,
        view_count -> Int4,
        seo_name -> Text,
        title -> Text,
        body -> Text,
        deleted -> Bool,
    }
}

table! {
    note_history (id) {
        id -> Uuid,
        note_id -> Uuid,
        created -> Timestamptz,
        title -> Text,
        body -> Text,
    }
}

table! {
    note_link (id) {
        id -> Uuid,
        left -> Uuid,
        right -> Uuid,
        click_count -> Int4,
    }
}

table! {
    user (id) {
        id -> Uuid,
        name -> Text,
        password -> Text,
    }
}

table! {
    user_token (id) {
        id -> Uuid,
        user_id -> Uuid,
        ip -> Text,
        last_used -> Timestamptz,
        active -> Bool,
    }
}

joinable!(note -> user (user_id));
joinable!(note_history -> note (note_id));
joinable!(user_token -> user (user_id));

allow_tables_to_appear_in_same_query!(note, note_history, note_link, user, user_token,);
