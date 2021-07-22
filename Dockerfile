FROM rust:alpine

WORKDIR /build
COPY . /build
ENV SQLX_OFFLINE true
RUN apk add musl-dev openssl-dev
RUN cargo install sqlx-cli
RUN cargo sqlx prepare --check
RUN cargo build --release

FROM scratch
COPY --from=0 /build/target/release/attendance-rs /attendance-rs
CMD ["/attendance-rs"]
