#!/bin/bash

source .env

vsock-proxy $PROXY_PORT kms.us-east-2.amazonaws.com 443