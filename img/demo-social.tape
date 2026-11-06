# Run from the repository root after cargo build --locked.
# Keep the original 16px text size in a smaller, 2:1 viewport.
# Menlo 16px: 96 columns x 22 rows on the verified VHS build.
Output "./img/demo-social.gif"
Set Shell zsh
Set FontFamily "Menlo"
Set FontSize 16
Set LetterSpacing 0
Set LineHeight 1
Set Width 960
Set Height 480
Set Padding 8
Set WindowBar Colorful
Set WindowBarSize 24
Set Theme "Tomorrow Night Eighties"
Set CursorBlink false
Set Framerate 8
Set TypingSpeed 100ms

Hide
Type@1ms "source ./img/setup.sh demo-social && capture_app"
Enter
Sleep 2s
Show

# Browse and filter, with shorter pauses than the full demo.
Sleep 1s
Type "j"
Sleep 500ms
Type "j"
Sleep 500ms
Type "k"
Sleep 500ms
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

# Pin 3024 Night and compare it with Aardvark Blue.
Type "p"
Sleep 500ms
Type "j"
Sleep 500ms
Type "c"
Sleep 2s

# Palette, Shell, Git, Logs, TUI, and Code.
Type "]"
Sleep 1s
Type "]"
Sleep 1s
Type "]"
Sleep 1s
Type "]"
Sleep 1s
Type "]"
Sleep 2s

# Show color values and contrast, then finish on the comparison.
Type "d"
Sleep 2s
Type@300ms "jjj"
Sleep 1s
Escape
Sleep 1s
Escape
Sleep 1s

# Let any in-flight canvas capture finish before quitting to the shell.
Hide
Sleep 1s
Type "q"
Sleep 500ms
