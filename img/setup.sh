# Sourced by the tapes from the repository root, inside VHS's private shell.
case "$1" in
    demo|demo-social|screenshots) ;;
    *) return 1 ;;
esac

capture_state_dir=$(mktemp -d "${TMPDIR:-/tmp}/asinus.XXXXXX") || return 1
mkdir -p "$capture_state_dir/bin" ./target/vhs ./img || return 1
printf '%s\n' "$capture_state_dir" > "./target/vhs/$1-state-dir"
stty size > "$capture_state_dir/terminal-size.txt"

# Keep clipboard writes in the same isolated directory as application state.
for capture_command in pbcopy wl-copy xclip; do
    printf '%s\n' '#!/bin/sh' 'exec /bin/cat > "$VHS_CAPTURE_COPY"' \
        > "$capture_state_dir/bin/$capture_command" || return 1
    chmod 700 "$capture_state_dir/bin/$capture_command" || return 1
done

capture_app() {
    ASINUS_STATE_PATH="${capture_state_dir:?}/marks.toml" \
    ASINUS_WORKSPACE_PATH="$capture_state_dir/workspace.toml" \
    VHS_CAPTURE_COPY="$capture_state_dir/clipboard.txt" \
    PATH="$capture_state_dir/bin:$PATH" \
    NO_COLOR= COLORTERM=truecolor \
    WAYLAND_DISPLAY=vhs-capture \
        ./target/release/asinus
}
