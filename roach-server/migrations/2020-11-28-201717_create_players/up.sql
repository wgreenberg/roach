create table players (
    id serial primary key not null,
    name text not null,
    elo integer not null,
    token_hash text unique not null
)
