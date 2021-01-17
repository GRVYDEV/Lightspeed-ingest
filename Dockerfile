FROM rust:latest as builder

WORKDIR /rust/src/

RUN apt update && apt install -y musl-tools
RUN rustup target add x86_64-unknown-linux-musl

COPY . .

RUN RUST_BACKTRACE=1 cargo install --target x86_64-unknown-linux-musl --path .
RUN mkdir /data

FROM scratch

COPY --from=builder /usr/local/cargo/bin/lightspeed-ingest /usr/local/bin/lightspeed-ingest
COPY --from=builder /data /data

EXPOSE 8084

WORKDIR /data

CMD ["lightspeed-ingest"]
