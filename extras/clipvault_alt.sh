#!/usr/bin/env bash
# Alternates between listing entries and sending an entry straight to `wl-copy` based on whether
# input was given. Also useful for exiting pickers without making a selection, as it will just list
# entries again, not override the clipboard with an empty item.

if [ "$1" = "" ]; then
    clipvault list
else
    clipvault get <<< "$1" | wl-copy
fi
