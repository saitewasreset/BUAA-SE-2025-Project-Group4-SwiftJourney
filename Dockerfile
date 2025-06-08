# backend

FROM rust AS chef 
WORKDIR /app
RUN apt-get update
RUN apt-get -y install libpq-dev lld
# https://www.aloxaf.com/2018/09/reduce_rust_size/
RUN apt-get -y install binutils
RUN wget https://github.com/upx/upx/releases/download/v4.2.4/upx-4.2.4-amd64_linux.tar.xz
RUN tar -xf upx-4.2.4-amd64_linux.tar.xz
# We only pay the installation cost once, 
# it will be cached from the second build onwards
RUN cargo install cargo-chef 

FROM chef AS planner
COPY ./backend ./
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY ./backend ./
ENV RUSTFLAGS="-C link-arg=-fuse-ld=lld"
RUN cargo build --release --bin api
RUN strip target/release/api
RUN ./upx-4.2.4-amd64_linux/upx --best target/release/api

# frontend

# build stage
FROM node:alpine as build-deps
WORKDIR /app
RUN --mount=type=bind,source=./SwiftJourney-Front-End/package.json,target=package.json \
    --mount=type=bind,source=./SwiftJourney-Front-End/package-lock.json,target=package-lock.json \
    --mount=type=cache,target=/root/.npm \
    npm ci

FROM build-deps as fbuild

COPY ./SwiftJourney-Front-End ./
RUN npm run build

# production stage
FROM debian:stable-slim
RUN apt-get update
RUN apt-get -y install libpq5
COPY --from=builder /app/target/release/api /app/swift-journey-backend
COPY --from=fbuild /app/dist /static

CMD ["/app/swift-journey-backend"]