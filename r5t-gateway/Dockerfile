FROM rust:latest as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo install --path ./r5t-gateway

FROM debian:buster-slim
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/r5t-gateway /usr/local/bin/r5t-gateway
CMD ["r5t-gateway"]