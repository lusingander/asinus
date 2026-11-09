# Configuration

Use `config.toml` to customize the Asinus UI.

## Configuration File Location

Asinus selects the configuration file in this order.

1. The path set in `ASINUS_CONFIG_FILE`.
2. `$XDG_CONFIG_HOME/asinus/config.toml` if `XDG_CONFIG_HOME` is nonempty.
3. `$HOME/.config/asinus/config.toml`.

Asinus reads only the selected file. Relative paths are resolved from the current working directory. If the selected default file is missing, Asinus uses its built-in settings.

An empty `ASINUS_CONFIG_FILE`, a missing file specified by that variable, or an unreadable or invalid configuration file prevents startup. Unknown settings are reported as errors.

## Create a Configuration

Save the following example as `config.toml` in your current directory. It sets the colors of selected items and turns off reverse video.

```toml
[ui.style.selection]
fg = "#1e1e2e"
bg = "#89b4fa"
modifiers = []
```

Run Asinus with this file.

```sh
ASINUS_CONFIG_FILE=./config.toml asinus
```

All settings are optional. Add only the values you want to change.

## UI Style

`[ui.style]` controls the application UI. Preview samples and color swatches use the colors of the theme being inspected.

Each setting below is a table under `[ui.style]`. Use `fg` for the foreground color and `bg` for the background color. When omitted, `bg` inherits from `base`, and `fg` uses the listed color or inherits from `base`.

| Setting | Default | Use |
| --- | --- | --- |
| `base` | `"default"` for both colors | Main text and background |
| `heading` | Bold | Screen and panel headings |
| `focus` | `"cyan"`, bold | Active panels and controls |
| `selection` | Reverse video | Selected rows and menu items |
| `key` | `"cyan"`, bold in Help | Shortcut keys in Help and footers |
| `muted` | Base colors | Descriptions beside shortcut keys in footers |
| `condition_key` | `"green"` | Keys in search conditions |
| `search_match` | `"yellow"`, bold | Matching text in theme names |
| `comparison_diff` | `"yellow"` | Difference labels in Color details |
| `notice` | `"blue"` | Informational notices |
| `success` | `"green"` | Success notices |
| `warning` | `"yellow"` | Warning notices |
| `error` | `"red"`, bold for search input errors | Error notices and search input errors |

Color values are strings. Use `"default"` for the terminal's default color or `"#RRGGBB"` for an RGB color. ANSI color names are `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, and `white`, plus their `bright_` forms such as `bright_cyan`.

Styles other than `base` also accept a `modifiers` array with `"bold"`, `"dim"`, `"underline"`, and `"reverse"`. An explicit array replaces the default attributes, and `[]` clears them. Omit the array to keep the default attributes.

For editor validation, see the [configuration schema](https://github.com/lusingander/asinus/blob/master/config.json.schema).
