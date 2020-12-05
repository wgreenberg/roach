create table matches (
    id serial primary key not null,
    white_player_id integer not null references players(id),
    black_player_id integer not null references players(id),
    game_type text not null
)
