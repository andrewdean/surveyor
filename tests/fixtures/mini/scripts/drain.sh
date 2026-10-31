#!/usr/bin/env bash
source ./scripts/lib.sh

drain() {
  cargo run -p api
}

drain
