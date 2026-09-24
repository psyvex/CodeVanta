FROM rust:1.85-bookworm AS rust-toolchain

RUN apt-get update \
    && apt-get install -y --no-install-recommends python3 python3-venv nodejs npm \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace

CMD ["bash"]
