#!/bin/bash

nitro-cli run-enclave --eif-path vsock-sample-server.eif --memory 1024 --cpu-count 2 --enclave-cid 16 --enclave-name vsock-sample-server --debug-mode --attach-console
