FROM rust:latest as builder

WORKDIR /rust/src/

COPY . .

RUN cargo install --path .

FROM debian:buster-slim

COPY --from=builder /usr/local/cargo/bin/lightspeed-ingest /usr/local/bin/lightspeed-ingest

RUN mkdir /data

EXPOSE 8084

WORKDIR /data

CMD ["lightspeed-ingest"]
