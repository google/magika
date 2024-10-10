# Sample Dockerfile to build an image that ties together a Magika CLI,
# an ONNX Runtime, and a Magika model.
#
# It expects the root of the repository as build context:
# $ docker build -f go/docker/Dockerfile -t magika-go:latest .
#
# Then, to list the content type of the files in the current directory:
# docker run --rm --name magika-go --entrypoint magika -v $PWD:$PWD:ro -w $PWD magika-go:latest *

# Build stage for ONNX Runtime and magika.
FROM golang:latest AS build

# Work in a clean temp directory.
WORKDIR /tmp

# Download, check, and install ONNX Runtime (https://onnxruntime.ai/) in
# /opt/onnxruntime.
# Releases are located at https://github.com/microsoft/onnxruntime/releases.
# We need the SDK (/include) for compiling, and the library (/lib) for inference.
ARG ONNX_NAME=onnxruntime
ARG ONNX_ARCH=linux-x64
ARG ONNX_VERSION=1.19.2
ARG ONNX_FULLNAME=${ONNX_NAME}-${ONNX_ARCH}-${ONNX_VERSION}
ARG ONNX_TARBALL=${ONNX_FULLNAME}.tgz
ARG ONNX_DIGEST=eb00c64e0041f719913c4080e0fed7d9963dc3aa9b54664df6036d8308dbcd33

RUN curl -sL -O https://github.com/microsoft/${ONNX_NAME}/releases/download/v${ONNX_VERSION}/${ONNX_TARBALL} \
    && echo "${ONNX_DIGEST}  ${ONNX_TARBALL}" > checksum.txt \
    && sha256sum -c checksum.txt \
    && tar -xzf ${ONNX_TARBALL} -C /opt \
    && ln -s /opt/${ONNX_FULLNAME} /opt/onnxruntime

# Retrieve the magika go code from the build context, and generate the cli.
COPY go go/
WORKDIR go/cli
ARG CGO_ENABLED=1
ARG CGO_CFLAGS=-I/opt/onnxruntime/include
RUN go build -tags onnxruntime -ldflags="-linkmode=external -extldflags=-L/opt/onnxruntime/lib" .

# Final stage, copy resources from the build.
FROM debian:latest

# Add the ONNX Runtime.
COPY --from=build /opt/onnxruntime/lib /opt/onnxruntime/lib
ENV LD_LIBRARY_PATH=/opt/onnxruntime/lib

# Add the Magika model.
COPY assets/models/standard_v2_1 /opt/magika/standard_v2_1/
ENV MAGIKA_MODEL_PATH=/opt/magika/standard_v2_1

# Add the CLI.
COPY --from=build /tmp/go/cli/cli /usr/local/bin/magika
