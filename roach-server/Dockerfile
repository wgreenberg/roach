FROM rust:latest

COPY . /usr/src/roach

WORKDIR /usr/src/roach/roach-server
RUN cargo install --path .
RUN cargo install diesel_cli
CMD ["./entrypoint.sh"]
