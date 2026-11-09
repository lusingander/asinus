# Compatibility

Asinus targets macOS and Linux. Windows is not currently supported.

## Terminal

Use a terminal with True Color (24-bit RGB) support and at least **60 columns × 22 rows**. Larger windows give previews more room. Some panels fill the screen at smaller sizes, and preview content may be clipped.

The surrounding UI uses your terminal's colors by default. Preview samples use the selected theme's RGB colors. Text attributes such as bold and dim depend on terminal settings. To customize the UI, see [Configuration](configuration.md).

## Clipboard

Copy requires the following command on `PATH`:

| Environment | Command |
| --- | --- |
| macOS | `pbcopy` |
| Linux / Wayland | `wl-copy` |
| Linux / X11 | `xclip` |

Linux clipboard access uses `WAYLAND_DISPLAY` and `DISPLAY` to select the available commands. In a remote session, these commands run on the machine hosting Asinus.

## Keyboard Input

Your terminal must forward `F1` and modified key combinations to Asinus. If a key opens a terminal shortcut instead, adjust the terminal's key settings. See [Keybindings](keybindings.md) for available alternatives.
