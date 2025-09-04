# Generic Dockerfile for building any backend microservice binary
# Usage: docker build -f backend/Dockerfile.ms --build-arg BIN=user_api ./backend

FROM rust:1 AS builder
WORKDIR /usr/src/app

RUN apt-get update \
    && apt-get -y install --no-install-recommends libpq-dev lld binutils wget xz-utils ca-certificates upx-ucl \
    && rm -rf /var/lib/apt/lists/*

COPY . .
ENV RUSTFLAGS="-C link-arg=-fuse-ld=lld"

# Build only the requested binary
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/src/app/target \
    cargo build --release

# Declare BIN after base layers so cache can be shared across services
ARG BIN
ENV BIN=${BIN}

RUN --mount=type=cache,target=/usr/src/app/target \
    mkdir -p /usr/src/app/bin \
    && cp target/release/${BIN} /usr/src/app/bin/app
# && strip /usr/src/app/bin/app || true \
# && upx --best /usr/src/app/bin/app || true

FROM debian:stable-slim AS runtime
RUN apt-get update \
    && apt-get -y install --no-install-recommends libpq5 ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/app/bin/app /usr/local/bin/app
ENV RUST_LOG=info
EXPOSE 8080 23333
CMD ["/usr/local/bin/app"]
