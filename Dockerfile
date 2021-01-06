FROM rust:latest as builder

WORKDIR /rust/src/

COPY . .

RUN cargo install --path .

FROM debian:buster-slim

COPY --from=builder /usr/local/cargo/bin/lightspeed-ingest /usr/local/bin/lightspeed-ingest

EXPOSE 8084

CMD ["lightspeed-ingest"]
