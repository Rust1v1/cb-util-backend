FROM docker.io/rust:1.88-trixie AS builder

# Set the working directory inside the container
WORKDIR /build

RUN rustup component add rustfmt

# Copy the Cargo.toml and Cargo.lock files first.
# This allows Docker to cache the dependencies layer if they haven't changed.
COPY Cargo.toml Cargo.lock ./

COPY src ./src

RUN cargo fmt --check
# RUN cargo clippy --locked

RUN cargo build --release --locked

FROM docker.io/debian:trixie-slim AS backend
ARG DEFAULT_MONGO_PORT=27017
WORKDIR /app

RUN printf "[default.databases.mongo]\n" > Rocket.toml
RUN printf "mongodb://mongo:${DEFAULT_MONGO_PORT}\n" >> Rocket.toml

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/cb-util-backend .

ENTRYPOINT ["/app/cb-util-backend"]
