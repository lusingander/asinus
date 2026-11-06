# Explore & Compare

Explore shows the selected theme, called **Current**. Pin a theme to keep it as a reference while browsing other candidates, then use Compare to view Pin and Current side by side or stacked vertically.

![Comparing two themes](https://raw.githubusercontent.com/lusingander/asinus/master/img/compare-horizontal.png)

The pin stays available when you change search conditions. You can hide the theme list to give previews more room.

## Preview Samples

| Sample | Content |
| --- | --- |
| Palette | Foreground, background, ANSI colors, and text attributes |
| Shell | A prompt, file listings, and command output |
| Git | Diffs, commits, and branches |
| Logs | Timestamps, log levels, and messages |
| TUI | Lists, tables, selections, and bars |
| Code | A Rust syntax sample |

All samples are built in. They show how the theme's colors work together; actual applications may use different color assignments.

The preview selector offers Auto, Small, Medium, and Large content sizes. Auto chooses a size for the available area. Fixed sizes keep the same content when resizing. Content extending beyond the preview is clipped. Compare uses the same content size for both themes.

Previewing does not apply a theme to your terminal. Use [Theme Details](theme-details.md) to inspect colors and find source links.

## Operations

In Explore / Compare:

| Keys | Action |
| --- | --- |
| `↑` / `↓` | Select the previous / next theme |
| `p` | Pin the selected theme, replace the pin, or unpin it if already pinned |
| `c` | Toggle Compare; requires a pin |
| `v` | Toggle side-by-side / stacked comparison |
| `z` | Show or hide the theme list |
| `[` / `]` | Select the previous / next preview sample |
| `P` | Open the preview selector |

In the preview selector:

| Keys | Action |
| --- | --- |
| `Tab` | Switch between Sample and Content |
| `←` / `→` | Change the selected value |
| `Enter` | Apply the preview settings |

See [Keybindings](../reference/keybindings.md) for all controls.
