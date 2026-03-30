# Stage 1: Build Rust backend
FROM rust:1.83 AS rust-builder
WORKDIR /app
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates/ crates/
RUN cargo build --release -p ts-server

# Stage 2: Build Svelte frontend
FROM node:22-alpine AS svelte-builder
WORKDIR /app/dashboard
COPY dashboard/package*.json ./
RUN npm ci
COPY dashboard/ ./
RUN npm run build

# Stage 3: Final image
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app

COPY --from=rust-builder /app/target/release/ts-server .
COPY --from=svelte-builder /app/dashboard/build ./dashboard/build

RUN mkdir -p data

EXPOSE 3200

CMD ["./ts-server"]
