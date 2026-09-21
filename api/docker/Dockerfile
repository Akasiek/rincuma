# syntax=docker/dockerfile:1

FROM rust:1.96-alpine AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --locked --release

FROM alpine:3.24 AS runtime

RUN apk add --no-cache ca-certificates \
    && addgroup --system --gid 10001 rincuma \
    && adduser --system --disabled-password --no-create-home \
        --uid 10001 --ingroup rincuma rincuma \
    && mkdir --parents /var/lib/rincuma \
    && chown rincuma:rincuma /var/lib/rincuma

COPY --from=builder /app/target/release/rincuma-api /usr/local/bin/rincuma-api

ENV DATA_DIR=/var/lib/rincuma \
    HOST=0.0.0.0 \
    PORT=7878

USER rincuma

EXPOSE 7878

ENTRYPOINT ["rincuma-api"]
