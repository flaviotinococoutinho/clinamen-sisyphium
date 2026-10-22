#!/bin/sh
# Searches the repository for forbidden terms (products, brands, engines, production tools and
# engineering doctrines). Exits with 1 when any occurrence is found. The list itself, the
# bibliography and the continuous-integration configuration are outside the search.
set -u
cd "$(dirname "$0")/.." || exit 2
hits=$(grep -rnwF -f scripts/verba-vetita.txt crates hosts docs README.md 2>/dev/null \
  | grep -v '^docs/referencias.md:' || true)
if [ -n "$hits" ]; then
  printf '%s\n' "$hits"
  exit 1
fi
printf 'verba vetita: 0 ocorrências\n'
