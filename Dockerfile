# syntax=docker/dockerfile:1

ARG PYTHON_VERSION=3.11
FROM python:${PYTHON_VERSION}-slim as base

# 1. Create a dedicated non-root user and group
RUN groupadd -r magika && useradd -r -g magika magika

WORKDIR /magika

# 2. Change ownership of the working directory
RUN chown magika:magika /magika

# 3. Install Magika (Pinned version and cache disabled for a smaller image)
ARG MAGIKA_VERSION=1.0.3
RUN pip install --no-cache-dir magika==${MAGIKA_VERSION}

# 4. Switch to the non-root user
USER magika

# 5. Add a basic healthcheck
HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
  CMD magika --version || exit 1

ENTRYPOINT ["magika"]