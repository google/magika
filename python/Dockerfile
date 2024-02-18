# syntax=docker/dockerfile:1

ARG PYTHON_VERSION=3.11
FROM python:${PYTHON_VERSION}-slim as base

WORKDIR /magika

RUN --mount=type=cache,target=/root/.cache/pip \
    pip install magika

ENTRYPOINT ["magika"]
