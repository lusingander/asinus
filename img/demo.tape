# Run from the repository root after cargo build --locked.
# Menlo 16px, no letter spacing: 120 columns x 36 rows on the verified VHS build.
Output "./img/demo.gif"
Set Shell zsh
Set FontFamily "Menlo"
Set FontSize 16
Set LetterSpacing 0
Set LineHeight 1
Set Width 1190
Set Height 760
Set Padding 12
Set WindowBar Colorful
Set WindowBarSize 30
Set Theme "Tomorrow Night Eighties"
Set CursorBlink false
Set Framerate 10
Set TypingSpeed 100ms

# Isolate marks, workspace, and clipboard writes.
Hide
Type@1ms "source ./img/setup.sh demo && capture_app"
Enter
Sleep 2s
Show

# Browse: changing the selection immediately changes the palette.
Sleep 1s
Type "j"
Sleep 500ms
Type "j"
Sleep 500ms
Type "k"
Sleep 1s

# Filter: Appearance = Dark, Text contrast = Medium.
Type "f"
Sleep 1s
Right
Sleep 500ms
Tab
Sleep 200ms
Tab
Sleep 200ms
Tab
Sleep 200ms
Tab
Sleep 200ms
Tab
Sleep 500ms
Right
Sleep 200ms
Right
Sleep 1s
Enter
Sleep 1s

# Compare 3024 Night against the next matching theme, Aardvark Blue.
Type "p"
Sleep 500ms
Type "j"
Sleep 500ms
Type "c"
Sleep 2s

# Keep both themes fixed while showing all six preview samples.
# Palette is already visible; continue through Shell, Git, Logs, TUI, and Code.
Type "]"
Sleep 1500ms
Type "]"
Sleep 1500ms
Type "]"
Sleep 1500ms
Type "]"
Sleep 1500ms
Type "]"
Sleep 2s

# Inspect Foreground, Background, and an ANSI color in the comparison.
Type "d"
Sleep 2s
Type "j"
Sleep 300ms
Type "j"
Sleep 300ms
Type "j"
Sleep 500ms
Escape
Sleep 1s
Escape
Sleep 1s

# Search by query.
Type "/"
Sleep 1s
Type@200ms "one"
Sleep 1s
Enter
Sleep 1s
Escape
Sleep 1s

# Stop recording before quitting, so the shell never appears in the GIF.
Hide
Type "q"
Sleep 500ms
