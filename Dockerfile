FROM rust:alpine AS build-stage

RUN apk update
RUN apk add cmake make musl-dev g++ perl

WORKDIR /build
COPY Cargo.toml ./
COPY src ./src
COPY test-files ./test-files
# Run tests before release build
RUN cargo test --release
RUN cargo build --release

# Build image from scratch
FROM scratch
LABEL org.opencontainers.image.source="https://github.com/pcvolkmer/mv64e-kafka-to-rest-gateway"
LABEL org.opencontainers.image.licenses="AGPL-3.0-or-later"
LABEL org.opencontainers.image.description="Fetch MV64e Kafka records with DNPM V2.1 payload and send it to DNPM:DIP"

COPY --from=build-stage /build/target/release/mv64e-kafka-to-rest-gateway .
USER 65532
EXPOSE 3000
CMD ["./mv64e-kafka-to-rest-gateway"]
