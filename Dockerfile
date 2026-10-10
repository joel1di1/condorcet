# syntax=docker/dockerfile:1
# Compiles natively on the build machine and cross-compiles to the target arch (no slow emulation).
FROM --platform=$BUILDPLATFORM ghcr.io/rust-cross/cargo-zigbuild:latest AS build
ARG TARGETARCH
ARG TAILWIND_VERSION=v4.3.3
WORKDIR /app
RUN case "$(uname -m)" in x86_64) a=x64;; *) a=arm64;; esac \
 && curl -sSfL -o /usr/local/bin/tailwindcss https://github.com/tailwindlabs/tailwindcss/releases/download/$TAILWIND_VERSION/tailwindcss-linux-$a \
 && chmod +x /usr/local/bin/tailwindcss
RUN case "$TARGETARCH" in amd64) t=x86_64-unknown-linux-musl;; arm64) t=aarch64-unknown-linux-musl;; esac \
 && echo $t > /target && rustup target add $t
COPY . .
RUN tailwindcss -i static/input.css -o static/app.css --minify
RUN --mount=type=cache,target=/usr/local/cargo/registry --mount=type=cache,target=/app/target \
    cargo zigbuild --release --target $(cat /target) \
 && cp target/$(cat /target)/release/condorcet /condorcet \
 && mkdir /data

FROM scratch
COPY --from=build /condorcet /condorcet
COPY --from=build --chown=1000:1000 /data /data
USER 1000
ENV DATABASE_PATH=/data/condorcet.db PORT=3000
EXPOSE 3000
CMD ["/condorcet"]
