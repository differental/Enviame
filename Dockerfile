# syntax=docker/dockerfile:1

ARG RUST_VERSION=1.91.0
ARG APP_NAME=enviame

FROM rust:${RUST_VERSION}-alpine AS build
ARG APP_NAME
WORKDIR /app

RUN apk add --no-cache clang lld musl-dev git openssl-dev pkgconfig openssl-libs-static

ENV SQLX_OFFLINE=true
RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=assets,target=assets \
    --mount=type=bind,source=static_src,target=static_src \
    --mount=type=bind,source=.sqlx,target=.sqlx \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=bind,source=askama.toml,target=askama.toml \
    --mount=type=bind,source=build.rs,target=build.rs \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
cargo build --locked --profile release-prod && \
cp ./target/release-prod/$APP_NAME /bin/server


FROM alpine:3.18 AS final


ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser
USER appuser

COPY --from=build /bin/server /bin/

EXPOSE 3000

CMD ["/bin/server"]
