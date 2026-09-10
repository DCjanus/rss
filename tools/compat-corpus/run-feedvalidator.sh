#!/usr/bin/env bash

set -euo pipefail

readonly REPOSITORY="https://github.com/w3c/feedvalidator.git"
readonly REVISION="9ce274c9db93796b8ab2a44952b9da80811bf765"
readonly CACHE_DIR="target/compat-corpus"
readonly CHECKOUT="${CACHE_DIR}/feedvalidator"
readonly REPORT="tools/compat-corpus/progress/feedvalidator.json"
readonly CRATE_REVISION="$(git rev-parse HEAD)"

mkdir -p "${CACHE_DIR}"

if [[ ! -d "${CHECKOUT}/.git" ]]; then
    git clone --filter=blob:none --no-checkout "${REPOSITORY}" "${CHECKOUT}"
fi

if ! git -C "${CHECKOUT}" cat-file -e "${REVISION}^{commit}" 2>/dev/null; then
    git -C "${CHECKOUT}" fetch --depth=1 origin "${REVISION}"
fi

git -C "${CHECKOUT}" checkout --detach --quiet "${REVISION}"

cargo run --quiet \
    --manifest-path tools/compat-corpus/Cargo.toml \
    --bin structural \
    -- \
    --root "${CHECKOUT}" \
    --include testcases/rss \
    --include testcases/rss20 \
    --mode feedvalidator \
    --source-repository "https://github.com/w3c/feedvalidator" \
    --source-revision "${REVISION}" \
    --crate-revision "${CRATE_REVISION}" \
    --report "${REPORT}"

printf 'Report: %s\n' "${REPORT}"

