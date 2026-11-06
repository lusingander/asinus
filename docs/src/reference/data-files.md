# Data Files

Asinus manages two TOML files:

| File | Contents | Path override |
| --- | --- | --- |
| `marks.toml` | Favorites and ignored themes | `ASINUS_STATE_PATH` |
| `workspace.toml` | Saved filters, last applied conditions, preview sample and content size, comparison direction | `ASINUS_WORKSPACE_PATH` |

## Locations

The default directory is `$XDG_DATA_HOME/asinus` when `XDG_DATA_HOME` is an absolute path. Otherwise, Asinus uses `$HOME/.local/share/asinus`. This applies on both macOS and Linux.

Set either path override to choose a different file. Relative overrides are resolved from the current working directory:

```sh
ASINUS_STATE_PATH=./marks.toml \
ASINUS_WORKSPACE_PATH=./workspace.toml \
asinus
```

## Read-Only Mode

If either file is invalid or cannot be read or locked at startup, Asinus reports the reason and opens in read-only mode. You can browse and change conditions for that session, but favorites, ignored themes, saved filters, and session settings cannot be saved.

Check the reported file and directory permissions, or close another Asinus instance using the same files, then restart.
