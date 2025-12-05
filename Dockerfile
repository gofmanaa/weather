FROM rust:1.90.0-slim-bullseye AS chef
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Build dependencies (cached layer)
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Build application
COPY . .
RUN cargo build --release --locked

ENTRYPOINT ["bash"]

FROM debian:bullseye-slim AS runtime

RUN apt-get update -y \
    && apt-get install -y --no-install-recommends ca-certificates \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/weather /app/weather

ENTRYPOINT ["./weather"]
#CMD ["weather"]
