create table matches (
    id integer primary key not null,
    white_player_id integer not null,
    black_player_id integer not null,
    game_type text not null,
    foreign key (white_player_id) references players(id),
    foreign key (white_player_id) references players(id)
)
