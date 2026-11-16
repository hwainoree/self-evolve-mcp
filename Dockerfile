FROM rust:1.100-trixie AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --locked -j2

FROM debian:trixie-slim AS runtime
RUN groupadd --gid 10001 mcp \
    && useradd --uid 10001 --gid 10001 --no-create-home --shell /usr/sbin/nologin mcp
COPY --from=builder /app/target/release/self-evolve-mcp /usr/local/bin/self-evolve-mcp
USER 10001:10001
ENV BIND_ADDR=0.0.0.0:8080
EXPOSE 8080
STOPSIGNAL SIGTERM
ENTRYPOINT ["/usr/local/bin/self-evolve-mcp"]
