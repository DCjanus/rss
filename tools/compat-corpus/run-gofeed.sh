#!/usr/bin/env bash

set -euo pipefail

readonly GOFEED_REPOSITORY="https://github.com/mmcdole/gofeed.git"
readonly GOFEED_REVISION="253ddbe673480b9bd9abdd114755cde33f12afc4"
readonly CACHE_DIR="target/compat-corpus"
readonly GOFEED_DIR="${CACHE_DIR}/gofeed"
readonly REPORT="tools/compat-corpus/progress/gofeed.json"
readonly CRATE_REVISION="$(git rev-parse HEAD)"

mkdir -p "${CACHE_DIR}"

if [[ ! -d "${GOFEED_DIR}/.git" ]]; then
    git clone --filter=blob:none --no-checkout "${GOFEED_REPOSITORY}" "${GOFEED_DIR}"
fi

if ! git -C "${GOFEED_DIR}" cat-file -e "${GOFEED_REVISION}^{commit}" 2>/dev/null; then
    git -C "${GOFEED_DIR}" fetch --depth=1 origin "${GOFEED_REVISION}"
fi

git -C "${GOFEED_DIR}" checkout --detach --quiet "${GOFEED_REVISION}"

cargo run --quiet \
    --manifest-path tools/compat-corpus/Cargo.toml \
    --bin rss-compat-corpus \
    -- \
    --gofeed "${GOFEED_DIR}/testdata/parser/rss" \
    --source-revision "${GOFEED_REVISION}" \
    --crate-revision "${CRATE_REVISION}" \
    --report "${REPORT}"

printf 'Report: %s\n' "${REPORT}"
