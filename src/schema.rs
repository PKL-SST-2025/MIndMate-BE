// @generated automatically by Diesel CLI.

diesel::table! {
    help_requests (id) {
        id -> Nullable<Integer>,
        user_id -> Integer,
        name -> Text,
        email -> Text,
        message -> Text,
        created_at -> Timestamp,
    }
}

diesel::table! {
    journals (id) {
        id -> Nullable<Integer>,
        user_id -> Integer,
        title -> Text,
        content -> Text,
        created_at -> Timestamp,
    }
}

diesel::table! {
    moods (id) {
        id -> Nullable<Integer>,
        user_id -> Integer,
        mood -> Text,
        intensity -> Integer,
        description -> Nullable<Text>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    psychologist_requests (id) {
        id -> Nullable<Integer>,
        user_id -> Integer,
        name -> Text,
        email -> Text,
        message -> Text,
        preferred_time -> Nullable<Text>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    users (id) {
        id -> Integer,
        username -> Text,
        email -> Text,
        password -> Text,
        settings -> Nullable<Text>,
        created_at -> Timestamp,
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
    users,
);
