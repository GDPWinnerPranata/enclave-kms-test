#!/bin/bash

source .env

echo "[i] Cleaning up ..."
ssh $USERNAME@$INSTANCE 'rm -rfv ~/rs/*'

echo "[i] Creating directory ~/rs/target/x86_64-unknown-linux-musl/release ..."
ssh $USERNAME@$INSTANCE 'mkdir -p ~/rs/target/x86_64-unknown-linux-musl/release'

./tf.sh

echo "[i] Copying kmstool_enclave_cli ..."
scp kmstool_enclave_cli $USERNAME@$INSTANCE:~/rs/

echo "[i] Copying libnsm.so ..."
scp libnsm.so $USERNAME@$INSTANCE:~/rs/