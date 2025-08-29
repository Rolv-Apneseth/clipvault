#!/usr/bin/env bash
# A `wofi` wrapper, with image previews.

tmp_dir="${XDG_CACHE_HOME:-$HOME/.cache}/clipvault/thumbs"
rm -rf "$tmp_dir"
mkdir -p "$tmp_dir"

read -r -d '' prog << EOF
/^[0-9]+\s<meta http-equiv=/ { next }
match(\$0, /^([0-9]+)\s\[\[\sbinary.*(jpg|jpeg|png|bmp|webp|tif|gif)/, grp) {
    image = grp[1]"."grp[2]
    system("echo " grp[1] " | clipvault get >$tmp_dir/"image)
    print "text:"grp[1]"\\t:img:$tmp_dir/"image
    next
}
1
EOF

choice=$(gawk <<< "$(clipvault list)" "$prog" | wofi -I --dmenu -Dimage_size=100 -Dynamic_lines=true -d -k /dev/null)
if [ "$choice" = "" ]; then
    exit 1
fi

if [ "${choice::5}" = "text:" ]; then
    choice="${choice:5}"
fi

echo "$choice" | clipvault get | wl-copy
