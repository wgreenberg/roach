table! {
    matches (id) {
        id -> Integer,
        white_player_id -> Integer,
        black_player_id -> Integer,
        game_type -> Text,
        status -> Text,
    }
}

table! {
    players (id) {
        id -> Integer,
        name -> Text,
        elo -> Integer,
        token_hash -> Text,
    }
}

allow_tables_to_appear_in_same_query!(
    matches,
    players,
);
