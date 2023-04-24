// @generated automatically by Diesel CLI.

diesel::table! {
    tasks (id) {
        id -> Text,
        name -> Text,
        description -> Text,
        status -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}
