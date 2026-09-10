#!/usr/bin/env bash

set -euo pipefail

readonly REPOSITORY="https://github.com/kurtmckee/feedparser.git"
readonly REVISION="a22c5521cbb109871f1a2318948581901bd47e26"
readonly CACHE_DIR="target/compat-corpus"
readonly CHECKOUT="${CACHE_DIR}/feedparser"
readonly REPORT="tools/compat-corpus/progress/feedparser.json"
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
    --include tests/wellformed/rss \
    --include tests/wellformed/rdf \
    --mode feedparser-wellformed \
    --source-repository "https://github.com/kurtmckee/feedparser" \
    --source-revision "${REVISION}" \
    --crate-revision "${CRATE_REVISION}" \
    --report "${REPORT}"

printf 'Report: %s\n' "${REPORT}"

