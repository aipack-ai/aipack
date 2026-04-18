# syntax=docker/dockerfile:1
ARG RUST_VERSION=1.94.0
ARG ALPINE_VERSION=3.23

FROM rust:${RUST_VERSION}-alpine${ALPINE_VERSION} AS build
WORKDIR /src

COPY . .

RUN apk update && apk add --no-cache clang lld musl-dev git
RUN cargo build --locked --release


FROM alpine:${ALPINE_VERSION} AS final

WORKDIR /app
ARG UID=1000
RUN adduser \
    --disabled-password \
    --home "/app" \
    --shell "/bin/sh" \
    --no-create-home \
    --uid "${UID}" \
    aipack
USER aipack

# Copy the executable from the "build" stage.
COPY --from=build /src/target/release/aip /bin

# What the container should run when it is started.
ENTRYPOINT ["/bin/aip"]
