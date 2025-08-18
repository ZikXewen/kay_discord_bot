FROM rust:1.89-slim AS builder

RUN apt update && \
    apt install -y curl cmake gcc git libssl-dev pkg-config && \
    rm -rf /var/lib/apt/lists/*

RUN USER=root cargo new --bin kay
WORKDIR /kay

COPY ./Cargo.lock ./Cargo.toml ./
RUN cargo build --release && \
    rm src/*.rs && \
    rm ./target/release/deps/kay*

COPY ./src ./src
RUN cargo build --release

####################################

FROM debian:bookworm-slim AS runner

RUN apt update && \
    apt install -y curl && \
    rm -rf /var/lib/apt/lists/* && \
    curl -L https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_linux -o /usr/local/bin/yt-dlp && \
    chmod +x /usr/local/bin/yt-dlp

COPY --from=builder /kay/target/release/kay /usr/src/kay
ENTRYPOINT ["/usr/src/kay"]
