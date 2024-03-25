# Use the official Rust image as the base image
FROM rust:1.77 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --package discord_bridgebot

### stage 2
# Create a new lightweight image with just the binary
FROM debian:bookworm-slim

RUN apt update && apt install -y libpq5

# Set the working directory inside the container
WORKDIR /app

# Copy the binary from the builder stage to the final image
COPY --from=builder /app/target/release/discord_bridgebot /app/discord_bridgebot

# Command to run your application
CMD ["/app/discord_bridgebot"]
