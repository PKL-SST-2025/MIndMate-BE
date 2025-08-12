// @generated automatically by Diesel CLI.

diesel::table! {
    help_requests (id) {
        id -> Int4,
        user_id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 255]
        email -> Varchar,
        message -> Text,
        created_at -> Timestamp,
    }
}

diesel::table! {
    journals (id) {
        id -> Int4,
        user_id -> Int4,
        #[max_length = 500]
        title -> Varchar,
        content -> Text,
        created_at -> Timestamp,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    moods (id) {
        id -> Int4,
        user_id -> Int4,
        date -> Date,
        #[max_length = 50]
        mood -> Varchar,
        #[max_length = 10]
        emoji -> Varchar,
        notes -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    psychologist_requests (id) {
        id -> Int4,
        user_id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 255]
        email -> Varchar,
        message -> Text,
        #[max_length = 255]
        preferred_time -> Nullable<Varchar>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    token_blacklist (id) {
        id -> Int4,
        token -> Text,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        #[max_length = 255]
        username -> Varchar,
        #[max_length = 255]
        email -> Varchar,
        #[max_length = 255]
        password -> Varchar,
        settings -> Nullable<Text>,
        age -> Nullable<Int4>,
        #[max_length = 50]
        gender -> Nullable<Varchar>,
        avatar -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::joinable!(help_requests -> users (user_id));
diesel::joinable!(journals -> users (user_id));
diesel::joinable!(moods -> users (user_id));
diesel::joinable!(psychologist_requests -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    help_requests,
    journals,
    moods,
    psychologist_requests,
    token_blacklist,
    users,
);
