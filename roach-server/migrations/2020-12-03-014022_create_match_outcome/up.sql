create table match_outcomes (
    id integer primary key not null,
    match_id integer not null,
    winner_id integer,
    loser_id integer,
    is_draw boolean not null,
    is_fault boolean not null,
    comment text not null,
    game_string text not null,
    foreign key (match_id) references matches(id)
    foreign key (winner_id) references players(id)
    foreign key (loser_id) references players(id)
)
