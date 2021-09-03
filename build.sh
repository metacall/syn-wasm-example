#!/bin/sh

docker build --build-arg BUILD_TYPE=release -t metacall/syn-wasm .
#docker run --rm -it metacall/syn-wasm bash
