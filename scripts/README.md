# Scripts

`import_themes.py` downloads the pinned VHS catalog from `assets/vhs/source.json` and updates the bundled catalog and license. Run from the repository root:

```sh
python3 scripts/import_themes.py
```

Validate the bundled catalog without network access:

```sh
python3 scripts/import_themes.py --check
```
