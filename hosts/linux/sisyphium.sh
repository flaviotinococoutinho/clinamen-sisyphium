#!/bin/sh
# Native capability probes; no package installation or desktop activation.
set -eu
script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd -P)
platform=linux
state_dir="${XDG_STATE_HOME:-$HOME/.local/state}/sisyphium"
schedule_dir="${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user"
measure() {
    if [ -z "$energy" ]; then
        energy=255
        for supply in /sys/class/power_supply/*; do
            [ -r "$supply/status" ] && [ -r "$supply/capacity" ] || continue
            [ "$(cat "$supply/status")" = Discharging ] || continue
            percent=$(cat "$supply/capacity") || continue
            if decimal "$percent" 0 100; then energy=$((value * 255 / 100)); fi
            break
        done
    fi
    dimensions=
    if [ -n "${WAYLAND_DISPLAY:-}" ] && command -v swaymsg >/dev/null 2>&1; then
        dimensions=$(swaymsg -p -t get_outputs 2>/dev/null | awk '/Current mode:/ {split($3,d,"x"); print d[1],d[2]; exit}') || dimensions=
    elif [ -n "${DISPLAY:-}" ] && command -v xrandr >/dev/null 2>&1; then
        dimensions=$(xrandr --current 2>/dev/null | awk '/ connected/ {for(i=1;i<=NF;i++) if($i ~ /^[0-9]+x[0-9]+[+]/) {split($i,d,/[x+]/); print d[1],d[2]; exit}}') || dimensions=
    fi
    measured_width=320; measured_height=180
    set -f
    set -- $dimensions
    set +f
    if [ "$#" = 2 ] && decimal "$1" 1 65535; then
        candidate_width=$value
        if decimal "$2" 1 65535 && [ $((candidate_width * value)) -le 33554432 ]; then measured_width=$candidate_width; measured_height=$value; fi
    fi
}
check_applicator() {
    if [ -z "$backend" ]; then
        if [ -n "${WAYLAND_DISPLAY:-}" ] && command -v swww >/dev/null 2>&1; then backend=swww
        elif [ -n "${DISPLAY:-}" ] && command -v feh >/dev/null 2>&1; then backend=feh
        else fail 'missing wallpaper backend: feh with DISPLAY or swww with WAYLAND_DISPLAY'; fi
    fi
    require "$backend"
    case $backend in feh) [ -n "${DISPLAY:-}" ] || fail 'feh requires DISPLAY';; swww) [ -n "${WAYLAND_DISPLAY:-}" ] || fail 'swww requires WAYLAND_DISPLAY and a running swww daemon';; esac
}
apply_image() {
    case $backend in
        feh) feh --no-fehbg --bg-fill -- "$state_dir/tabula.png" || fail 'feh failed';;
        swww) swww img --transition-type none "$state_dir/tabula.png" || fail 'swww failed; ensure its daemon is running';;
    esac
}
schedule_arguments() {
    printf 'ExecStart='
    for arg in /bin/sh "$script_dir/sisyphium.sh" apply --binary "$binary" --state-dir "$state_dir" --interval "$interval" --lingua "$lingua"; do unit_arg "$arg"; printf ' '; done
    for pair in energy width height backend; do
        case $pair in energy) arg=$explicit_energy;; width) arg=$explicit_width;; height) arg=$explicit_height;; backend) arg=$backend;; esac
        if [ -n "$arg" ]; then unit_arg "--$pair"; printf ' '; unit_arg "$arg"; printf ' '; fi
    done
    printf '\n'
}
install_schedule() {
    require systemctl
    for name in sisyphium.service sisyphium.timer; do
        [ ! -e "$schedule_dir/$name" ] && [ ! -L "$schedule_dir/$name" ] || fail "schedule exists; uninstall first: $name"
    done
    mkdir -p "$schedule_dir" || fail 'cannot create user unit directory'
    for name in sisyphium.service sisyphium.timer; do
        (set -C; while IFS= read -r line; do case $line in @ARGUMENTS@) schedule_arguments;; @INTERVAL@) printf 'OnActiveSec=%ss\nOnUnitInactiveSec=%ss\n' "$interval" "$interval";; *) printf '%s\n' "$line";; esac; done < "$script_dir/$name.in" > "$schedule_dir/$name") || fail 'cannot create unit'
    done
    systemctl --user daemon-reload || fail 'systemctl daemon-reload failed'
    systemctl --user enable --now sisyphium.timer || fail 'systemctl enable failed'
}
uninstall_schedule() {
    found=false
    for name in sisyphium.service sisyphium.timer; do
        file="$schedule_dir/$name"
        if [ -e "$file" ] || [ -L "$file" ]; then
            [ ! -L "$file" ] && grep -q '^# sisyphium-owned$' "$file" || fail "unrecognized schedule: $name"
            found=true
        fi
    done
    [ "$found" = true ] || return 0
    require systemctl
    systemctl --user disable --now sisyphium.timer || fail 'systemctl disable failed; units preserved'
    # Stop any active own service before removing its definition.
    systemctl --user stop sisyphium.service || fail 'systemctl stop failed; units preserved'
    for name in sisyphium.service sisyphium.timer; do
        file="$schedule_dir/$name"; [ ! -f "$file" ] || rm -- "$file" || fail 'cannot remove unit'
    done
    systemctl --user daemon-reload || fail 'systemctl daemon-reload failed'
}
. "$script_dir/../communis.sh"
