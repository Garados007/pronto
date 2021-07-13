table! {
    fast_token (id) {
        id -> Uuid,
        token -> Text,
        server_id -> Uuid,
        game -> Text,
        lobby -> Text,
        created_at -> Timestamp,
        updated_at -> Nullable<Timestamp>,
    }
}

table! {
    server (id) {
        id -> Uuid,
        last_seen -> Timestamp,
        token -> Text,
        created_at -> Timestamp,
        updated_at -> Nullable<Timestamp>,
    }
}

table! {
    server_game (id) {
        id -> Uuid,
        name -> Text,
        uri -> Text,
        rooms -> Int4,
        max_rooms -> Nullable<Int4>,
        clients -> Int4,
        game_info_id -> Uuid,
        created_at -> Timestamp,
        updated_at -> Nullable<Timestamp>,
    }
}

table! {
    server_info (id) {
        id -> Uuid,
        name -> Text,
        uri -> Text,
        developer -> Bool,
        fallback -> Bool,
        full -> Bool,
        maintenance -> Bool,
        max_clients -> Nullable<Int4>,
        server_id -> Uuid,
        created_at -> Timestamp,
        updated_at -> Nullable<Timestamp>,
    }
}

joinable!(fast_token -> server (server_id));
joinable!(server_game -> server_info (game_info_id));
joinable!(server_info -> server (server_id));

allow_tables_to_appear_in_same_query!(
    fast_token,
    server,
    server_game,
    server_info,
);
