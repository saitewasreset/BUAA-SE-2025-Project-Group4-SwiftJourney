# build stage
FROM rust AS builder
WORKDIR /usr/src/swift-journey-backend
RUN apt-get update
RUN apt-get -y install libpq-dev lld
# https://www.aloxaf.com/2018/09/reduce_rust_size/
RUN apt-get -y install binutils
RUN wget https://github.com/upx/upx/releases/download/v4.2.4/upx-4.2.4-amd64_linux.tar.xz
RUN tar -xf upx-4.2.4-amd64_linux.tar.xz

COPY ./apps ./apps
COPY ./base ./base
COPY ./id_macro ./id_macro
COPY ./migration ./migration
COPY ./shared ./shared
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
ENV RUSTFLAGS="-C link-arg=-fuse-ld=lld"

RUN cargo build --release --bin api
RUN strip target/release/api
RUN ./upx-4.2.4-amd64_linux/upx --best target/release/api

# production stage
FROM debian:stable-slim
RUN apt-get update
RUN apt-get -y install libpq5
COPY --from=builder /usr/src/swift-journey-backend/target/release/api /usr/local/bin/api
CMD ["api"]