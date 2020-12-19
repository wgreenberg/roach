FROM rust:latest

COPY . /usr/src/roach

WORKDIR /usr/src/roach
RUN cargo install --path hive --bin cli-engine
RUN cargo install --path roach-client
WORKDIR /usr/src/roach/roach-client
CMD ["./entrypoint.sh"]
