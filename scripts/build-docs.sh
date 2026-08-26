#!/usr/bin/env bash
set -euo pipefail

npm ci
python3 scripts/check-docs-leakage.py docs
npm run docs:build
python3 scripts/check-docs-leakage.py docs/.vitepress/dist
