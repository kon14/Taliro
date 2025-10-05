# ==============================================================================
# Build Stage
# ==============================================================================
FROM rust:1.89-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /usr/src/app

# Copy manifest files (layer caching)
COPY Cargo.toml Cargo.lock ./
COPY application/Cargo.toml ./application/
COPY common/Cargo.toml ./common/
COPY domain/Cargo.toml ./domain/
COPY infrastructure/Cargo.toml ./infrastructure/
COPY macros/Cargo.toml ./macros/
COPY main/Cargo.toml ./main/
COPY presentation/Cargo.toml ./presentation/

# Create dummy source files (enable dependency caching)
RUN mkdir -p application/src common/src domain/src infrastructure/src macros/src main/src presentation/src && \
    echo "fn main() {}" > main/src/main.rs && \
    echo "// dummy" > application/src/lib.rs && \
    echo "// dummy" > common/src/lib.rs && \
    echo "// dummy" > domain/src/lib.rs && \
    echo "// dummy" > infrastructure/src/lib.rs && \
    echo "// dummy" > macros/src/lib.rs && \
    echo "// dummy" > presentation/src/lib.rs

# Build dependency cache
RUN cargo fetch

# Copy real source files
RUN rm -rf application/src common/src domain/src infrastructure/src macros/src main/src presentation/src
COPY application/ ./application/
COPY common/ ./common/
COPY domain/ ./domain/
COPY infrastructure/ ./infrastructure/
COPY macros/ ./macros/
COPY main/ ./main/
COPY presentation/ ./presentation/

# Build the application
ENV RUSTFLAGS="-A warnings"
RUN cargo build --release --bin Taliro


# ==============================================================================
# Runtime Stage
# ==============================================================================
FROM debian:bookworm-slim as runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    procps \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -r -s /bin/false -m -d /app appuser

# Set working directory
WORKDIR /app

# Copy the compiled binary
COPY --from=builder /usr/src/app/target/release/Taliro /usr/local/bin/taliro

# Create data directory for taliro storage
RUN mkdir -p /app/data && chown -R appuser:appuser /app

# Switch to non-root user
USER appuser

# Expose ports (HTTP, P2P)
EXPOSE 4000 5000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
    CMD pgrep taliro || exit 1

# Set default environment variables
ENV STORAGE_DB_PATH=
ENV HTTP_API_PORT=4000
ENV HTTP_API_BASE_URL=http://localhost:4000
ENV HTTP_MASTER_KEY_SECRET=
ENV NETWORK_LISTEN_ADDRESS=/ip4/0.0.0.0/tcp/0
ENV NETWORK_INIT_PEERS=;
ENV NETWORK_IDENTITY_KEY_PAIR=
ENV RUST_LOG=info
ENV NODE_PORT=8080
ENV P2P_PORT=9000

# Run the taliro node
CMD ["taliro"]
