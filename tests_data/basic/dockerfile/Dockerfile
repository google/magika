# syntax=docker/dockerfile:1

ARG PYTHON_VERSION=3.11
FROM python:${PYTHON_VERSION}-slim as base

WORKDIR /magika

# This requires buildx
# RUN --mount=type=cache,target=/root/.cache/pip \
#     pip install magika

RUN pip install magika

ENTRYPOINT ["magika"]
