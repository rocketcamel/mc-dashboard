#!/usr/bin/env bash

services=("api" "frontend")

if [ -z "$1" ]; then
  echo "usage: $0 <service>"
  exit 1
fi

start_api() {
  cd api && cargo run
}

start_frontend() {
  cd frontend && bun dev
}

case "$1" in
  "api")
    start_api
    ;;
  "frontend")
    start_frontend
    ;;
  *)
    echo "unknown service $1"
    echo "available services: ${services[*]}"
    exit 1
    ;;
esac
