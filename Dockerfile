FROM rust:1.87 as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /usr/src/app/target/release/xxx_chat_service /usr/local/bin/

CMD ["xxx_chat_service"]
