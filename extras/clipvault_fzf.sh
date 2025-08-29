#!/usr/bin/env bash
# Shows preview for the highlighted entry, and only copies to clipboard if a value was selected.

case "$1" in
    preview)
        if echo "$2" | grep -vqP '^\d+\t\[\[ binary data .*\]\]'; then
            # Show preview of non-binary data
            echo "$2" | clipvault get
        elif echo "$2" | grep -vqP '^\d+\t\[\[ binary data .*image.*\]\]'; then
            # Show preview of binary, non-image data
            echo "$2" | cut -f2-
        else
            # Show preview of images using sixel
            echo "$2" | clipvault get | chafa -f sixel -s "${FZF_PREVIEW_COLUMNS}x${FZF_PREVIEW_LINES}"
        fi
        ;;
    *)
        exec clipvault list \
            | fzf -d $'\t' --with-nth 2 --preview "$(realpath "$0") preview {}" \
            | {
                read -r output && clipvault get <<< "$output" | wl-copy
            }
        ;;
esac
