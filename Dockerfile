FROM rust

RUN mkdir -p /op/prime-search
WORKDIR /opt/prime-search
COPY ./ ./
RUN cargo build --release

CMD ["./target/release/prime-search"]
