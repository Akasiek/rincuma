# syntax=docker/dockerfile:1

FROM rust:1.96-alpine3.24 AS api-build

RUN apk add --no-cache build-base
WORKDIR /build/api
COPY api/Cargo.toml api/Cargo.lock ./
COPY api/src ./src
RUN --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=/build/api/target \
  cargo build --locked --release --features migration-cli --bins \
  && mkdir /artifacts \
  && cp target/release/rincuma-api target/release/rincuma-migrate /artifacts/

FROM node:24-alpine3.24 AS web-build

ARG PNPM_VERSION=12.14.0
RUN npm install --global pnpm@${PNPM_VERSION}
WORKDIR /build/web
COPY web/package.json web/pnpm-lock.yaml ./
RUN --mount=type=cache,target=/pnpm/store \
  pnpm install --frozen-lockfile --store-dir=/pnpm/store --ignore-scripts
COPY web/ ./
RUN pnpm run build

FROM node:24-alpine3.24 AS runtime

RUN apk add --no-cache bash ca-certificates tini tzdata
WORKDIR /app/api
COPY --from=api-build /artifacts/ /usr/local/bin/
COPY --from=web-build /build/web/.output /app/web
COPY api/toasty ./toasty
COPY --chmod=755 docker/start.sh /usr/local/bin/rincuma-start
COPY docker/healthcheck.mjs /app/healthcheck.mjs

# Expose Nuxt and let its server proxy requests to the API on loopback.
ENV NODE_ENV=production \
  NITRO_HOST=0.0.0.0 \
  NITRO_PORT=3000 \
  NUXT_API_BASE=http://127.0.0.1:7878 \
  HOST=127.0.0.1 \
  PORT=7878 \
  COOKIE_SECURE=true

USER node
EXPOSE 3000
HEALTHCHECK --interval=10s --timeout=6s --start-period=60s --retries=5 \
  CMD ["node", "/app/healthcheck.mjs"]
# Tini forwards signals and reaps orphaned child processes.
ENTRYPOINT ["/sbin/tini", "--", "/usr/local/bin/rincuma-start"]
