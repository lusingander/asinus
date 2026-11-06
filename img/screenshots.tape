# Run from the repository root after cargo build --locked.
# Documentation images use one 120x36 terminal with the same styling as the demo.
# VHS requires a video output; discard this ignored intermediate after recording.
Output "./target/vhs/screenshots.gif"
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
Set TypingSpeed 10ms

Hide
Type@1ms "source ./img/setup.sh screenshots && capture_app"
Enter
Sleep 2s

# Start with a recognizable dark theme and explicitly fix all six samples to Large.
Type "/"
Ctrl+U
Type "Catppuccin Mocha"
Enter
Sleep 500ms
Type "RP"
Tab
Left
Enter
Sleep 500ms
Show

Screenshot "./img/preview-palette.png"
Sleep 300ms
Type "]"
Sleep 500ms
Screenshot "./img/preview-shell.png"
Sleep 300ms
Type "]"
Sleep 500ms
Screenshot "./img/preview-git.png"
Sleep 300ms
Type "]"
Sleep 500ms
Screenshot "./img/preview-logs.png"
Sleep 300ms
Type "]"
Sleep 500ms
Screenshot "./img/preview-tui.png"
Sleep 300ms
Type "]"
Sleep 500ms
Screenshot "./img/preview-code.png"
Sleep 300ms

# The picker and list visibility each need only one representative image.
Type "P"
Tab
Sleep 500ms
Screenshot "./img/preview-selection.png"
Sleep 300ms
Escape
Sleep 300ms
Type "z"
Sleep 500ms
Screenshot "./img/explore-list-hidden.png"
Sleep 300ms
Type "z"
Sleep 300ms

# Let detail panels and comparisons choose a fitting sample size automatically.
Type "P"
Tab
Right
Enter
Type "]d"
Sleep 500ms
Screenshot "./img/color-details.png"
Sleep 300ms
Escape
Sleep 300ms
Type "iZ"
Sleep 500ms
Screenshot "./img/theme-info.png"
Sleep 300ms
Type "Z"
Escape
Sleep 300ms

# Compare distinct palettes, retaining the pinned theme while searching for nord.
Type "p/"
Ctrl+U
Type "nord"
Enter
Sleep 500ms
Type "Rc]"
Sleep 500ms
Screenshot "./img/compare-horizontal.png"
Sleep 300ms
Type "v"
Sleep 500ms
Screenshot "./img/compare-vertical.png"
Sleep 300ms
Type "v"
Sleep 300ms
Type "z"
Sleep 500ms
Screenshot "./img/compare-list.png"
Sleep 300ms
Type "z"
Sleep 300ms
Type "djjj"
Sleep 500ms
Screenshot "./img/compare-colors.png"
Sleep 300ms
Escape
Sleep 300ms
Type "iZ"
Sleep 500ms
Screenshot "./img/compare-info.png"
Sleep 300ms
Type "Zl"
Sleep 500ms
Screenshot "./img/theme-credits.png"
Sleep 300ms
Escape
Sleep 300ms
Type "yjt"
Sleep 500ms
Screenshot "./img/copy.png"
Sleep 300ms
Escape
Sleep 300ms
Type "c"
Sleep 300ms

# Search and the structured filter editor are separate entry points.
Type "/"
Ctrl+U
Type "catpp"
Sleep 500ms
Screenshot "./img/search.png"
Sleep 300ms
Escape
Sleep 300ms
Type "f"
Right
Tab 5
Right 2
Sleep 500ms
Screenshot "./img/filters.png"
Sleep 300ms
Enter
Sleep 300ms

# Capture saving and managing named conditions, without success notifications.
Type "SDark / Medium"
Sleep 500ms
Screenshot "./img/save-filter.png"
Sleep 300ms
Enter
Sleep 500ms
Type "/"
Ctrl+U
Type "type:light"
Enter
Sleep 300ms
Type "SLight themes"
Enter
Sleep 500ms
Type "sj"
Sleep 500ms
Screenshot "./img/saved-filters.png"
Sleep 300ms
Escape
Sleep 300ms

# Show Pin, Favorite, and Ignore markers together with ignored themes visible.
Type "Rpmjjx"
Sleep 300ms
Type "/"
Ctrl+U
Type "ignore:all"
Enter
Sleep 300ms
Type "g"
Sleep 500ms
Screenshot "./img/marks.png"
Sleep 300ms

Hide
Sleep 1s
Type "q"
Sleep 500ms
