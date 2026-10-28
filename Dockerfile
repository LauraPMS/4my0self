FROM rust:1.80-alpine AS builder
RUN apk add --no-cache musl-dev
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY migrations ./migrations
COPY src ./src
ENV SQLX_OFFLINE=false
RUN cargo build --release

FROM alpine:3.20
RUN apk add --no-cache ca-certificates
WORKDIR /app
COPY --from=builder /app/target/release/personal-os-core /app/personal-os-core
COPY migrations /app/migrations

RUN adduser -D -u 1000 appuser
USER appuser

EXPOSE 8080
ENTRYPOINT ["/app/personal-os-core"]