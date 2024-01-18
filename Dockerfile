# Use the official Rust image as the base image
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --package rust_bridgebot

### stage 2
# Create a new lightweight image with just the binary
FROM debian:bookworm-slim

RUN apt update && apt install -y libpq5

# Set the working directory inside the container
WORKDIR /app

# Copy the binary from the builder stage to the final image
COPY --from=builder /app/target/release/rust_bridgebot /app/rust_bridgebot

# Command to run your application
CMD ["/app/rust_bridgebot"]
