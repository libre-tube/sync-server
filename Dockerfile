FROM ghcr.io/rust-lang/rust:alpine AS builder

ARG DATABASE_BACKEND=sqlite

COPY . .

RUN apk add sqlite-static sqlite-dev musl-dev
RUN cargo build --release --features "${DATABASE_BACKEND}"

FROM alpine:latest AS runner
WORKDIR /app

COPY --from=builder target/release/libretube-sync /app/libretube-sync-server

EXPOSE 8080
CMD ["./libretube-sync-server"]
