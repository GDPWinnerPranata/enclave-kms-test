#!/bin/bash

source .env

echo "[i] Building binaries ..."
./build-binaries.sh

echo "[i] Copying targets ..."
scp target/x86_64-unknown-linux-musl/release/vsock-server $USERNAME@$INSTANCE:~/rs/target/x86_64-unknown-linux-musl/release/
scp target/x86_64-unknown-linux-musl/release/vsock-client $USERNAME@$INSTANCE:~/rs/target/x86_64-unknown-linux-musl/release/

echo "[i] Copying Dockerfile ..."
scp Dockerfile.* $USERNAME@$INSTANCE:~/rs/

echo "[i] Copying .env ..."
scp .env $USERNAME@$INSTANCE:~/rs/

echo "[i] Copying scripts ..."
scp ./scripts/* $USERNAME@$INSTANCE:~/rs/