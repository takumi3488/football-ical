FROM node:24-slim AS web
ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
WORKDIR /app
RUN corepack enable
COPY . /app
RUN --mount=type=cache,id=pnpm,target=/pnpm/store pnpm install --frozen-lockfile
RUN pnpm run build

FROM rust:1 AS rust
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

FROM gcr.io/distroless/cc-debian12 AS crawler
WORKDIR /app
COPY --from=rust /usr/src/app/target/release/crawl /app/crawl
ENTRYPOINT ["/app/crawl"]

FROM gcr.io/distroless/cc-debian12 AS server
WORKDIR /app
COPY --from=web /app/dist /app/dist
COPY --from=rust /usr/src/app/target/release/server /app/server
ENTRYPOINT ["/app/server"]
