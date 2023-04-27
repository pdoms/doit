// @generated automatically by Diesel CLI.

diesel::table! {
    tasks (id) {
        id -> Varchar,
        name -> Varchar,
        description -> Text,
        status -> Varchar,
        due -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}
