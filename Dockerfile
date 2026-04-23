FROM ghcr.io/rust-lang/rust:alpine AS builder
WORKDIR /build-dir

RUN apk add sqlite-static sqlite-dev musl-dev

ARG DATABASE_BACKEND=sqlite
ARG TARGETPLATFORM # automatically assigned by Docker Buildx

COPY . .

# Cache compiled dependencies, taken from:
# - https://www.blacksmith.sh/blog/cache-is-king-a-guide-for-docker-layer-caching-in-github-actions
# - https://gist.github.com/noelbundick/6922d26667616e2ba5c3aff59f0824cd
# - https://stackoverflow.com/questions/58473606/cache-rust-dependencies-with-docker-build
RUN --mount=type=cache,target=/usr/local/cargo/registry,id=${TARGETPLATFORM}-${DATABASE_BACKEND} \
    --mount=type=cache,target=/build-dir/target,id=${TARGETPLATFORM}-${DATABASE_BACKEND} \
    cargo build --release --locked --no-default-features --features "${DATABASE_BACKEND}" \
    && mv target/release/libretube-sync libretube-sync

FROM alpine:latest AS runner
WORKDIR /app

COPY --from=builder /build-dir/libretube-sync /app/libretube-sync-server

EXPOSE 8080
CMD ["./libretube-sync-server"]
