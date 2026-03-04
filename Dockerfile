# Stage 1: Build frontend
FROM node:20-slim AS frontend-builder
WORKDIR /app/frontend
COPY frontend/package*.json ./
RUN npm ci
COPY frontend/ ./
RUN npm run build

# Stage 2: Build backend
FROM rust:1.82-slim AS backend-builder
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
WORKDIR /app/backend
COPY backend/Cargo.toml backend/Cargo.lock* ./
COPY backend/src/ ./src/
RUN cargo build --release

# Stage 3: Runtime
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
RUN useradd -r -s /bin/false sunrisewinds

WORKDIR /app
COPY --from=backend-builder /app/backend/target/release/sunrisewinds ./
COPY --from=frontend-builder /app/frontend/dist ./frontend/dist/

RUN mkdir -p /app/data && chown sunrisewinds:sunrisewinds /app/data
USER sunrisewinds

ENV RUST_LOG=sunrisewinds=info

EXPOSE 3000

CMD ["./sunrisewinds"]
