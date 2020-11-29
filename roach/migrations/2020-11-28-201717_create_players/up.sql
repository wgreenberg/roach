create table players (
    id integer primary key,
    name text not null,
    elo integer not null,
    token_hash text unique not null
)
