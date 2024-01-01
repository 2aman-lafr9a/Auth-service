# Use the latest version of the Rust base image
FROM rust:latest

# Set the working directory in the container to /my
WORKDIR /usr/src/my-app

# update the apt-get package manager
RUN apt-get update -y

# install the protoco compiler
RUN apt-get install protobuf-compiler -y

# Copy the Rust project files to the working directory
COPY . .

# Build the Rust app
RUN cargo build --release

# Set the command to run the Rust app
CMD cargo run