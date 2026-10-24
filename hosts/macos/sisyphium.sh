#!/bin/sh
# launchd and osascript contracts: hosts/README.md and docs/referencias.md.
set -eu
script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd -P)
platform=macos
state_dir="$HOME/Library/Application Support/sisyphium"
schedule="$HOME/Library/LaunchAgents/org.sisyphium.tabula.plist"
measure() {
    sw_vers -productVersion >/dev/null 2>&1 || :
    if [ -z "$energy" ]; then
        percent=$(pmset -g batt 2>/dev/null | awk '/discharging;/ {for(i=1;i<=NF;i++) if($i ~ /^[0-9]+%;$/) {sub(/%;/,"",$i); print $i; exit}}') || percent=
        if decimal "$percent" 0 100; then energy=$((value * 255 / 100)); else energy=255; fi
    fi
    dimensions=$(system_profiler SPDisplaysDataType 2>/dev/null | awk '/Resolution:/ {print $2, $4; exit}') || dimensions=
    measured_width=320; measured_height=180
    set -f
    set -- $dimensions
    set +f
    if [ "$#" = 2 ] && decimal "$1" 1 65535; then
        candidate_width=$value
        if decimal "$2" 1 65535 && [ $((candidate_width * value)) -le 33554432 ]; then measured_width=$candidate_width; measured_height=$value; fi
    fi
}
check_applicator() { require osascript; }
apply_image() {
    osascript - "$state_dir/tabula.png" <<'APPLESCRIPT' || fail 'osascript could not apply wallpaper'
on run argv
    set imagePath to POSIX file (item 1 of argv)
    tell application "System Events"
        repeat with screen in desktops
            set picture of screen to imagePath
        end repeat
    end tell
end run
APPLESCRIPT
}
# ProgramArguments is an XML array, with no shell between launchd and /bin/sh.
schedule_arguments() {
    for arg in /bin/sh "$script_dir/sisyphium.sh" apply --binary "$binary" --state-dir "$state_dir" --interval "$interval" --lingua "$lingua"; do printf '<string>%s</string>\n' "$(xml "$arg")"; done
    for pair in energy width height; do
        case $pair in energy) arg=$explicit_energy;; width) arg=$explicit_width;; height) arg=$explicit_height;; esac
        if [ -n "$arg" ]; then printf '<string>--%s</string><string>%s</string>\n' "$pair" "$arg"; fi
    done
}
install_schedule() {
    require launchctl; require plutil
    [ ! -e "$schedule" ] && [ ! -L "$schedule" ] || fail "schedule exists; uninstall first: $schedule"
    mkdir -p "$(dirname "$schedule")" || fail 'cannot create LaunchAgents directory'
    (set -C; while IFS= read -r line; do case $line in @ARGUMENTS@) schedule_arguments;; @INTERVAL@) printf '<integer>%s</integer>\n' "$interval";; *) printf '%s\n' "$line";; esac; done < "$script_dir/sisyphium.plist.in" > "$schedule") || fail 'cannot create schedule'
    plutil -lint "$schedule" >/dev/null || fail 'invalid generated plist; schedule retained for inspection'
    launchctl bootstrap "gui/$(id -u)" "$schedule" || fail 'launchctl bootstrap failed; schedule retained for inspection'
}
uninstall_schedule() {
    [ -e "$schedule" ] || { [ ! -L "$schedule" ] || fail 'symbolic schedule'; return; }
    [ ! -L "$schedule" ] && grep -q '<!-- sisyphium-owned -->' "$schedule" || fail 'unrecognized schedule'
    require launchctl
    launchctl bootout "gui/$(id -u)" "$schedule" || fail 'launchctl bootout failed; schedule preserved'
    rm -- "$schedule" || fail 'cannot remove schedule'
}
. "$script_dir/../communis.sh"
