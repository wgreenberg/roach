FROM rust:latest

COPY . /usr/src/roach

WORKDIR /usr/src/roach/hive
RUN cargo install --path .
WORKDIR /usr/src/roach/roach-client
RUN cargo install --path .
CMD ["./entrypoint.sh"]
