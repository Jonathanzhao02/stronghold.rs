# Build Stage
FROM --platform=linux/amd64 rustlang/rust:nightly as builder

## Install build dependencies.
RUN apt-get update && \
    DEBIAN_FRONTEND=noninteractive apt-get install -y cmake clang

## Add source code to the build stage.
ADD . /stronghold.rs
WORKDIR /stronghold.rs/client/fuzz

## Build instructions
RUN cargo install cargo-fuzz
RUN make build
RUN ls

# Package Stage
FROM --platform=linux/amd64 ubuntu:20.04

## TODO: Change <Path in Builder Stage>
COPY --from=builder /stronghold.rs/client/fuzz /fuzz

