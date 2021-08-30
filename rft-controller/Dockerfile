# syntax=docker/dockerfile:1.2
FROM rust:latest as builder
WORKDIR /usr/src/app
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/src/app/target \
    cargo install --path ./rft-controller

FROM debian:buster-slim
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/rft-controller /usr/local/bin/rft-controller
CMD ["rft-controller"]