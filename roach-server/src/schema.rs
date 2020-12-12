table! {
    matches (id) {
        id -> Int4,
        white_player_id -> Int4,
        black_player_id -> Int4,
        game_type -> Text,
        winner_id -> Nullable<Int4>,
        loser_id -> Nullable<Int4>,
        is_draw -> Bool,
        is_fault -> Bool,
        time_started -> Timestamptz,
        time_finished -> Timestamptz,
        comment -> Text,
        game_string -> Text,
    }
}

table! {
    players (id) {
        id -> Int4,
        name -> Text,
        elo -> Int4,
        token_hash -> Text,
    }
}

allow_tables_to_appear_in_same_query!(
    matches,
    players,
);
