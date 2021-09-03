# Build stage
FROM rust:slim-bullseye AS build

# Install WASI
RUN rustup default nightly \
	&& cargo install cargo-wasi

# Prepare the workspace and build the project
WORKDIR /root/metacall-syn-wasm
COPY . .
ARG BUILD_TYPE
RUN cargo wasi build --${BUILD_TYPE}

# Test for build
RUN cargo install wasm-nm \
	&& wasm-nm -z ./target/wasm32-wasi/${BUILD_TYPE}/metacall_syn_wasm.wasm | grep dump_ast

# Main stage
FROM debian:bullseye-slim AS main

WORKDIR /root/metacall-syn-wasm

# Install dependencies
RUN apt-get update \
	&& apt-get install -y --no-install-recommends build-essential cmake ca-certificates git

# Clone and build MetaCall
RUN git clone --branch develop https://github.com/metacall/core \
	&& mkdir core/build && cd core/build \
	&& cmake \
		-DOPTION_BUILD_LOADERS_WASM=On \
		-DOPTION_BUILD_DETOURS=Off \
		-DOPTION_BUILD_SCRIPTS=Off \
		-DOPTION_BUILD_TESTS=Off \
		-DOPTION_BUILD_EXAMPLES=Off \
		.. \
	&& cmake --build . --target install \
	&& cd ../.. \
	&& rm -rf core

# Set up project with WASM binaries
ARG BUILD_TYPE
COPY --from=build /root/metacall-syn-wasm/target/wasm32-wasi/${BUILD_TYPE}/ /root/metacall-syn-wasm/

# Test file for the parser
RUN printf 'pub fn greet(name: &str) -> String {\n\tformat!("Hello, {}!", name)\n}\n' >> /root/test.rs

# TODO: Not working yet (needs a small core refactor)
RUN export LOADER_SCRIPT_PATH="/root/metacall-syn-wasm/" \
	&& printf 'package wasm metacall_syn_wasm.wasm\ncall dump_ast()\nexit' | metacallcli
