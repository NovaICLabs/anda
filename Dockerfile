FROM  ubuntu:24.04

RUN apt-get update \
    && apt-get install -y gcc g++ libc6-dev pkg-config libssl3 wget protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY target/x86_64-unknown-linux-gnu/release/anda_bot /app/
COPY output/Character.toml /app/
COPY output/Config.toml /app/
COPY ./.env /app/

RUN mkdir -p /app/object_store

EXPOSE 8080

CMD ["./anda_bot", "start-local"]