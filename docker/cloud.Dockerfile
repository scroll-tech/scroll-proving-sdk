# Use the official Rust image as the base image
FROM scrolltech/go-rust-builder:go-1.21-rust-nightly-2023-12-03 as builder

# Set the working directory
WORKDIR /usr/src/app

# Copy the entire project
COPY . .

# Build the project
RUN cargo build --release --example cloud

# Create a new stage with a minimal Ubuntu image
FROM ubuntu:20.04

# Install necessary dependencies
RUN apt-get update && apt-get install -y libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy the built binary from the builder stage
COPY --from=builder /usr/src/app/target/release/examples/cloud /usr/local/bin/cloud

# Set the entrypoint
ENTRYPOINT ["cloud"]