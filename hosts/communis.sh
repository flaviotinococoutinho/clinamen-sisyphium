#!/bin/sh
# Argument bounds follow tabula; scheduler escaping follows the native manuals.
set -eu
export LC_ALL=C
fail() { printf '%s\n' "sisyphium: $*" >&2; exit 1; }
usage_error() { printf '%s\n' "sisyphium: $*" >&2; exit 2; }
# Decimal syntax prevents shell arithmetic from interpreting octal or expressions.
decimal() {
    case $1 in ''|*[!0-9]*) return 1;; esac
    [ "${#1}" -le 10 ] || return 1
    value=$(printf '%s' "$1" | sed 's/^0*//'); value=${value:-0}
    [ "$value" -ge "$2" ] && [ "$value" -le "$3" ]
}
# A quoted argument is diagnostic text only; it is never evaluated.
show_command() {
    printf 'command:'
    for arg do printf " '%s'" "$(printf '%s' "$arg" | sed "s/'/'\\\\''/g")"; done
    printf '\n'
}
xml() { printf '%s' "$1" | sed 's/\&/\&amp;/g;s/</\&lt;/g;s/>/\&gt;/g;s/"/\&quot;/g;s/'"'"'/\&apos;/g'; }
# systemd performs specifier and environment expansion after tokenization.
unit_arg() { printf '"%s"' "$(printf '%s' "$1" | sed 's/\\/\\\\/g;s/"/\\"/g;s/%/%%/g;s/\$/$$/g')"; }
require() { command -v "$1" >/dev/null 2>&1 || fail "missing utility: $1"; }
absolute() { case $1 in /*) printf '%s' "$1";; *) printf '%s/%s' "$PWD" "$1";; esac; }
action=dry-run
binary="$script_dir/../../target/release/sisyphium-tabula"
interval=60; energy=; width=; height=; lingua=en; backend=
seen=' '
case ${1-} in dry-run|run|apply|install|uninstall) action=$1; shift;; esac
while [ "$#" -gt 0 ]; do
    key=$1; shift
    case $seen in *" $key "*) usage_error "duplicate option: $key";; esac
    seen="$seen$key "
    [ "$#" -gt 0 ] || usage_error "missing value: $key"
    argument=$1; shift
    case $key in
        --binary) binary=$argument;; --state-dir) state_dir=$argument;;
        --interval) decimal "$argument" 60 2147483647 || usage_error 'interval must be 60..2147483647 seconds'; interval=$value;;
        --energy) decimal "$argument" 0 255 || usage_error 'energy must be 0..255'; energy=$value;;
        --width|--height) decimal "$argument" 1 65535 || usage_error 'dimensions must be 1..65535'; if [ "$key" = --width ]; then width=$value; else height=$value; fi;;
        --lingua) case $argument in en|pt) lingua=$argument;; *) usage_error 'lingua must be en or pt';; esac;;
        --backend) [ "$platform" = linux ] || usage_error "unknown option: $key"; case $argument in feh|swww) backend=$argument;; *) usage_error 'backend must be feh or swww';; esac;;
        *) usage_error "unknown option: $key";;
    esac
done
[ -n "$binary" ] && [ -n "$state_dir" ] || usage_error 'paths must not be empty'
binary=$(absolute "$binary"); state_dir=$(absolute "$state_dir")
# Scheduled arguments preserve explicit measurements; automatic probes run each time.
explicit_energy=$energy; explicit_width=$width; explicit_height=$height
if [ "$action" = uninstall ]; then uninstall_schedule; exit 0; fi
[ -f "$binary" ] && [ -x "$binary" ] || fail "missing executable: $binary"
if [ "$action" = install ]; then
    # Native schedule formats cannot represent all Unix byte paths.
    require iconv
    for arg in "$binary" "$state_dir" "$script_dir"; do
        printf '%s' "$arg" | iconv -f UTF-8 -t UTF-8 >/dev/null 2>&1 || usage_error 'schedule paths must be UTF-8'
        if printf '%s' "$arg" | grep '[[:cntrl:]]' >/dev/null; then usage_error 'schedule paths must not contain control characters'; fi
    done
    install_schedule; exit 0
fi
measure
energy=${energy:-255}; width=${width:-$measured_width}; height=${height:-$measured_height}
[ $((width * height)) -le 33554432 ] || usage_error 'image area exceeds 33554432 pixels'
set -- "$binary" --state "$state_dir/mundus.hex" --output "$state_dir/tabula.png" --format png --energy "$energy" --width "$width" --height "$height" --lingua "$lingua"
show_command "$@"
[ "$action" != dry-run ] || exit 0
if [ "$action" = apply ]; then check_applicator; fi
mkdir -p "$state_dir" || fail 'cannot create state directory'
"$@" || fail 'tabula failed; wallpaper was not applied'
# Validate the publication contract before any desktop call.
[ -f "$state_dir/mundus.hex" ] && [ ! -L "$state_dir/mundus.hex" ] || fail 'missing or symbolic state'
[ "$(wc -c < "$state_dir/mundus.hex" | tr -d ' ')" = 32 ] || fail 'invalid state length'
LC_ALL=C grep -Eq '^[0-9a-fA-F]{32}$' "$state_dir/mundus.hex" || fail 'invalid state encoding'
[ -f "$state_dir/tabula.png" ] && [ ! -L "$state_dir/tabula.png" ] || fail 'missing or symbolic image'
[ "$(od -An -tx1 -N8 "$state_dir/tabula.png" | tr -d ' \n')" = 89504e470d0a1a0a ] || fail 'invalid PNG signature'
[ "$action" != apply ] || apply_image
