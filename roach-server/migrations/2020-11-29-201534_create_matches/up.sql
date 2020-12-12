create table matches (
    id serial primary key not null,
    white_player_id integer not null references players(id),
    black_player_id integer not null references players(id),
    game_type text not null,
    winner_id integer references players(id),
    loser_id integer references players(id),
    is_draw boolean not null,
    is_fault boolean not null,
    time_started timestamp with time zone not null,
    time_finished timestamp with time zone not null,
    comment text not null,
    game_string text not null
)
