FROM rust:1.51 as builder
WORKDIR /usr/src/node-template
COPY . .
RUN apt update && \
apt install -y cmake pkg-config libssl-dev git gcc build-essential git clang libclang-dev
RUN rustup update nightly && \
rustup target add wasm32-unknown-unknown --toolchain nightly && \
cargo install --path ./bin/node-template/node --locked

FROM phusion/baseimage:0.11
LABEL maintainer="chevdor@gmail.com"
LABEL description="This is the 2nd stage: a very small image where we copy the Substrate binary."

RUN mv /usr/share/ca* /tmp && \
	rm -rf /usr/share/*  && \
	mv /tmp/ca-certificates /usr/share/ && \
	useradd -m -u 1000 -U -s /bin/sh -d /substrate substrate && \
	mkdir -p /substrate/.local/share/node-template && \
	chown -R substrate:substrate /substrate/.local && \
	ln -s /substrate/.local/share/node-template /data

COPY --from=builder /usr/local/cargo/bin/node-template /usr/local/bin/node-template

# checks
RUN ldd /usr/local/bin/node-template && \
	/usr/local/bin/node-template --version

# Shrinking
RUN rm -rf /usr/lib/python* && \
	rm -rf /usr/bin /usr/sbin /usr/share/man

USER substrate
EXPOSE 30333 9933 9944 9615
VOLUME ["/data"]

CMD ["/usr/local/bin/node-template"]
