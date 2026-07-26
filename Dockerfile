FROM lukemathwalker/cargo-chef:latest-rust-alpine AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN apk add sqlite-static sqlite-dev openssl-libs-static openssl-dev
RUN cargo chef cook --release --recipe-path recipe.json

# Build application
COPY . .
ARG DATABASE_BACKEND=sqlite

RUN cargo build --release --locked --no-default-features --features "${DATABASE_BACKEND}"

FROM alpine:latest AS runner
WORKDIR /app

# required for connecting to YouTube for input data validation
RUN apk add ca-certificates

COPY --from=builder /app/target/release/libretube-sync /app/libretube-sync-server

EXPOSE 8080
CMD ["./libretube-sync-server"]
