FROM rust:1.82-alpine AS builder
RUN apk add --no-cache musl-dev openssl-dev pkgconf
WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
# Pre-cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release 2>/dev/null; rm -rf src
COPY . .
RUN touch src/main.rs && cargo build --release

FROM alpine:3.20
RUN apk add --no-cache ca-certificates tzdata openssl
WORKDIR /app
COPY --from=builder /app/target/release/starter-api .
COPY migrations ./migrations
RUN mkdir -p storage/photos
EXPOSE 8000
CMD ["./starter-api"]
