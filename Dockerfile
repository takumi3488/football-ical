FROM node:20-slim AS web
ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
WORKDIR /app
RUN corepack enable
COPY . /app
RUN --mount=type=cache,id=pnpm,target=/pnpm/store pnpm install --frozen-lockfile
RUN pnpm run build

FROM rust:1 AS server
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

FROM gcr.io/distroless/cc-debian12
WORKDIR /app
COPY --from=web /app/dist /app/dist
COPY --from=server /usr/src/app/target/release/server /app/server
ENTRYPOINT ["/app/server"]
