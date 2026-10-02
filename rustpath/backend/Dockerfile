FROM rust:1.99.0-slim-bookworm AS build
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations
COPY course.json ./
RUN cargo build --release --locked
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/* && useradd --uid 10001 --create-home app
COPY --from=build /app/target/release/rustpath-api /usr/local/bin/rustpath-api
USER 10001:10001
EXPOSE 3000
CMD ["rustpath-api"]
