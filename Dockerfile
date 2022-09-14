FROM rust:1.63.0-buster AS builder

# create new empty project
RUN cargo new --bin protohackers
WORKDIR /protohackers

# copy manifests
COPY Cargo.toml Cargo.lock ./

# build and cache dependencies
RUN cargo build --release
RUN rm src/*.rs

# copy source code over
COPY src/ src/

# remove cached objects and rebuild with real source code
RUN rm ./target/release/deps/protohackers*
RUN cargo build --release

FROM debian:buster-slim
COPY --from=builder /protohackers/target/release/protohackers /usr/local/bin
CMD ["protohackers"]
