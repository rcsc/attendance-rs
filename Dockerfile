# Stolen from https://gist.github.com/PurpleBooth/ec81bad0a7b56ac767e0da09840f835a
FROM rust:alpine

WORKDIR /build
COPY . /build
ENV SQLX_OFFLINE true
RUN apk add musl-dev openssl-dev
RUN cargo build --release

FROM scratch
COPY --from=0 /build/target/release/attendance-rs /attendance-rs
CMD ["/attendance-rs"]
