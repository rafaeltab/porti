# Use an official Rust runtime as a parent image
FROM rust:1.84-slim-bookworm

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Set the working directory inside the container
WORKDIR /usr/src/app

# Copy the entire source code
COPY . .

# Build the application
RUN cargo build --release --bin source_control

# Create a new smaller image for running the application
FROM debian:bookworm-slim AS runner

# Install required packages
RUN apt-get update && \
    apt-get install -y --no-install-recommends libssl-dev openssl ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Set the working directory
WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=0 /usr/src/app/target/release/source_control .

# Expose the port your app runs on - adjust as needed
EXPOSE 8080

# Command to run the executable
CMD ["./source_control"]
