# Build stage
FROM rust:1.75-alpine as builder

WORKDIR /app

# Install build dependencies
RUN apk add --no-cache \
    musl-dev \
    postgresql-client \
    openssl-dev

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY backend ./backend

# Build the application
WORKDIR /app/backend
RUN cargo build --release

# Runtime stage
FROM alpine:3.18

# Install runtime dependencies
RUN apk add --no-cache \
    ca-certificates \
    libssl3 \
    postgresql-client

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/backend/target/release/owo-backend /app/owo-backend

# Create non-root user
RUN addgroup -g 1000 owo && \
    adduser -D -u 1000 -G owo owo

USER owo

EXPOSE 3000

CMD ["./owo-backend"]
