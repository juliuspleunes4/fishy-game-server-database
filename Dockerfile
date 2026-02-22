# ------ build stage
FROM rust:1.89-slim AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY backend/ .

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
	libssl3 ca-certificates && \
	rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/backend .

# Copy and build the dependicies first, optimisation thing
#COPY backend/Cargo.toml backend/Cargo.lock ./
#RUN mkdir src && echo "fn main() {}" > src/main.rs
#RUN cargo build --release
#RUN rm -rf src

#COPY backend .
#RUN cargo build --release

# ------ runtime stage
#FROM debian:trixie-slim

#RUN apt-get update && apt-get install -y \
#    libssl3 ca-certificates binutils && \
#    rm -rf /var/lib/apt/lists/*

#WORKDIR /app

#COPY --from=builder /app/target/release/backend /app/

#RUN strip ./backend

CMD ["./backend"]
