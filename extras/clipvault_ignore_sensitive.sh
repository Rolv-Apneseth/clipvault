#!/usr/bin/env bash
# For those who don't have a build of `wl-clipboard` with the latest updates, or there
# are other MIME types you wish to ignore (replace `x-kde-passwordManagerHint` below).

if ! wl-paste --list-types | rg -q "x-kde-passwordManagerHint"; then
    clipvault store --store-sensitive < "${1:-/dev/stdin}"
fi
