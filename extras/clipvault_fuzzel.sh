#!/usr/bin/env bash
# A `fuzzel` wrapper, with image previews.
# Usage: /path/to/clipvault_fuzzel.sh

list=$(clipvault list)
thumbnails_dir="${XDG_CACHE_HOME:-$HOME/.cache}/clipvault/thumbs"

# Ensure thumbnail directory exists
[ -d "$thumbnails_dir" ] || mkdir -p "$thumbnails_dir"

# Delete thumbnails that are no longer in the DB
find "$thumbnails_dir" -type f | while IFS= read -r thumbnail; do
    id=$(basename "${thumbnail%.*}")
    if ! grep -q "^${id}\s\[\[ binary data" <<< "$list"; then
        rm "$thumbnail"
    fi
done

# Generates thumbnails (for matched image formats) for entries which don't already have one in the
# thumbnails directory, and returns entries ready to be displayed by `fuzzel`.
read -r -d '' prog << EOF
/^[0-9]+\s<meta http-equiv=/ { next }
match(\$0, /^([0-9]+)\s\[\[\sbinary.*(jpg|jpeg|png|bmp|webp|tif|gif)/, grp) {
    image = grp[1]"."grp[2]
    system("[[ -f ${thumbnails_dir}/"image" ]] || echo " grp[1] " | clipvault get >${thumbnails_dir}/"image)
    print \$0"\0icon\x1f${thumbnails_dir}/"image
    next
}
1
EOF

choice=$(echo "$list" | gawk "$prog" | fuzzel -d --placeholder "Clipboard" --counter --no-sort --with-nth 2)
exit_code=$?

# Custom keybinds (configured in your `fuzzel.ini`) used below for different actions.
# Delete all entries with `custom-0` (default: ALT+0)
if [ "$exit_code" -eq 19 ]; then
    confirmation=$(echo -e "N\ny" | fuzzel -d --placeholder "Delete history?" --lines 2)
    [ "$confirmation" == "y" ] && rm -rf "$thumbnails_dir" && clipvault clear
# Delete selected entry with `custom-1` (default: ALT+1)
elif [ "$exit_code" -eq 10 ]; then
    if [ "$choice" != "" ]; then
        id=$(echo "$choice" | cut -f1)
        clipvault delete "$id"
        find "$thumbnails_dir" -name "${id}.*" -delete
    fi
# Default case, copy entry to clipboard
else
    [ "$choice" = "" ] || echo "$choice" | clipvault get | wl-copy
fi
