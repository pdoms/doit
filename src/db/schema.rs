// @generated automatically by Diesel CLI.

diesel::table! {
    tasks (id) {
        id -> Text,
        name -> Text,
        description -> Text,
        status -> Text,
        due -> Nullable<Timestamp>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}
