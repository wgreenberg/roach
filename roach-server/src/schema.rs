table! {
    match_outcomes (id) {
        id -> Int4,
        match_id -> Int4,
        winner_id -> Nullable<Int4>,
        loser_id -> Nullable<Int4>,
        is_draw -> Bool,
        is_fault -> Bool,
        comment -> Text,
        game_string -> Text,
    }
}

table! {
    matches (id) {
        id -> Int4,
        white_player_id -> Int4,
        black_player_id -> Int4,
        game_type -> Text,
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

joinable!(match_outcomes -> matches (match_id));

allow_tables_to_appear_in_same_query!(
    match_outcomes,
    matches,
    players,
);
