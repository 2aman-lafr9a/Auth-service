# Use the latest version of the Rust base image
FROM rust:latest

# Set the working directory in the container to /my
WORKDIR /usr/src/my-app

# update the apt-get package manager
RUN apt-get update -y

# install the protoco compiler
RUN apt-get install protobuf-compiler -y

# Install the diesel CLI
RUN cargo install diesel_cli


# Copy the Rust project files to the working directory
COPY . .

# Build the Rust app
RUN cargo build --release

# Run the binary
CMD ["./target/release/auth-service"]