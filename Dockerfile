FROM rust:alpine

RUN \
  apk update && \
  apk upgrade && \
  apk add --no-cache bash ca-certificates coreutils curl alpine-sdk libffi-dev openssl-dev libc-dev git jq make openssh-client py-pip python3-dev