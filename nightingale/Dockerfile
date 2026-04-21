FROM rust:1.78-slim-bookworm AS builder

RUN apt-get update && apt-get install -y \
    pkg-config libssl-dev clang libclang-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build
# Copy the entire workspace so local path deps resolve
COPY . .

RUN cargo build -p nightingale --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ffmpeg yt-dlp ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/nightingale /usr/local/bin/nightingale

EXPOSE 2333
CMD ["nightingale"]
