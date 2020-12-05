create table match_outcomes (
    id serial primary key not null,
    match_id integer not null references matches(id),
    winner_id integer references players(id),
    loser_id integer references players(id),
    is_draw boolean not null,
    is_fault boolean not null,
    comment text not null,
    game_string text not null
)
