FROM rust:1.86 as builder

WORKDIR /app
COPY . .

RUN apt-get update && apt-get install -y pkg-config libssl-dev
RUN cargo build --release

FROM ubuntu:22.04

RUN apt-get update && apt-get install -y ca-certificates && apt-get clean

WORKDIR /app
COPY --from=builder /app/target/release/Worker-Cli ./Worker-Cli

EXPOSE 8080
CMD ["./Worker-Cli"]