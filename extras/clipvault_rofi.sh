#!/usr/bin/env bash
# Custom mode for `rofi`, including image preview.
# Usage: rofi -modi clipboard:/path/to/clipvault_rofi.sh -show clipboard -show-icons

rofi_list()
{
    list=$(clipvault list)

    # Ensure thumbnail directory exists
    thumbnails_dir="${XDG_CACHE_HOME:-$HOME/.cache}/clipvault/thumbs"
    [ -d "$thumbnails_dir" ] || mkdir -p "$thumbnails_dir"

    # Delete thumbnails that are no longer in the DB
    find "$thumbnails_dir" -type f | while IFS= read -r thumbnail; do
        item_id=$(basename "${thumbnail%.*}")
        if ! grep -q "^${item_id}\s\[\[ binary data" <<< "$list"; then
            rm "$thumbnail"
        fi
    done

    # Generates thumbnails (for matched image formats) for entries which don't already have one in
    # the thumbnails directory, and returns entries ready to be displayed by `rofi`.
    read -r -d '' prog << EOF
/^[0-9]+\s<meta http-equiv=/ { next }
match(\$0, /^([0-9]+)\s(\[\[\sbinary.*(jpg|jpeg|png|bmp|webp|tif|gif).*)/, grp) {
    image = grp[1]"."grp[3]
    system("[[ -f ${thumbnails_dir}/"image" ]] || echo " grp[1] " | clipvault get >${thumbnails_dir}/"image)
    print grp[2]"\000icon\037$thumbnails_dir/"image"\037info\037"grp[1]"\n"
    next
}
match(\$0, /^([0-9]+)\s(.*)/, grp) {
    print grp[2]"\000info\037"grp[1]"\n"
    next
}
1
EOF

    echo "$list" | gawk "$prog"
}

case $ROFI_RETV in
    # Display entries on startup
    0)
        echo -en "\000use-hot-keys\037true\n" # Enable custom keys
        rofi_list
        ;;
    # Handle regular select (kb-accept-entry, default <Enter>)
    1)
        if [ ! "$ROFI_INFO" = "" ]; then
            clipvault get "$ROFI_INFO" | wl-copy
        fi
        exit
        ;;
    # Handle entry deletion (kb-delete-entry, default <S-Delete>)
    3)
        if [ ! "$ROFI_INFO" = "" ]; then
            clipvault delete "$ROFI_INFO"
        fi
        rofi_list
        ;;
    # Handle custom keybind 1 (kb-custom-1) - delete all entries
    # For more about custom rofi keybinds, see <https://davatorium.github.io/rofi/current/rofi-keys.5>
    10)
        clipvault clear
        ;;
    # Handle custom keybind 2 (kb-custom-2) - attempt to write the text out directly using wtype
    11)
        if [ ! "$ROFI_INFO" = "" ]; then
            coproc {
                wtype "$(clipvault get "$ROFI_INFO")"
            }
        fi
        exit
        ;;
    # Notify of unhandled keybind return value
    *) notify-send "Clipboard: unhandled rofi return value $ROFI_RETV" ;;

        # Alternatively, just ignore unhandled keybinds
        # *) ;;
esac
