table! {
    match_outcomes (id) {
        id -> Integer,
        match_id -> Integer,
        winner_id -> Nullable<Integer>,
        loser_id -> Nullable<Integer>,
        is_draw -> Bool,
        is_fault -> Bool,
        comment -> Text,
        game_string -> Text,
    }
}

table! {
    matches (id) {
        id -> Integer,
        white_player_id -> Integer,
        black_player_id -> Integer,
        game_type -> Text,
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

joinable!(match_outcomes -> matches (match_id));

allow_tables_to_appear_in_same_query!(
    match_outcomes,
    matches,
    players,
);
