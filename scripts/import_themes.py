"""Import the pinned VHS catalog, preserving all upstream fields."""

import argparse
import json
from pathlib import Path
import re
from urllib.request import urlopen


DESTINATION = Path(__file__).resolve().parents[1] / "assets" / "vhs"
COLOR_FIELDS = (
    "foreground", "background",
    "black", "red", "green", "yellow", "blue", "magenta", "cyan", "white",
    "brightBlack", "brightRed", "brightGreen", "brightYellow",
    "brightBlue", "brightMagenta", "brightCyan", "brightWhite",
)


def validate_catalog(themes):
    if not isinstance(themes, list) or not themes:
        raise ValueError("Expected a non-empty theme catalog")
    names = set()
    for index, theme in enumerate(themes, start=1):
        if not isinstance(theme, dict):
            raise ValueError(f"Theme {index}: expected an object")
        name = theme.get("name")
        if not isinstance(name, str) or not name or name != name.strip():
            raise ValueError(f"Theme {index}: expected a non-empty, trimmed name")
        if name in names:
            raise ValueError(f"Duplicate theme name: {name}")
        names.add(name)
        metadata = theme.get("meta")
        if not isinstance(metadata, dict) or type(metadata.get("isDark")) is not bool:
            raise ValueError(f"{name}: meta.isDark must be a Boolean")
        fields = COLOR_FIELDS + tuple(
            field for field in ("cursor", "selection") if field in theme
        )
        for field in fields:
            value = theme.get(field)
            if not isinstance(value, str) or not re.fullmatch(r"#[0-9a-fA-F]{6}", value):
                raise ValueError(f"{name}: {field} must be a #RRGGBB color")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--check", action="store_true",
        help="validate the bundled catalog without network access or file changes",
    )
    args = parser.parse_args()
    try:
        source = json.loads((DESTINATION / "source.json").read_text(encoding="utf-8"))
        if args.check:
            themes = json.loads((DESTINATION / "themes.json").read_text(encoding="utf-8"))
            validate_catalog(themes)
            if len(themes) != source["theme_count"]:
                raise ValueError("Theme count does not match source.json")
        else:
            repository = source["repository"].removeprefix("https://github.com/")
            base_url = f"https://raw.githubusercontent.com/{repository}/{source['revision']}"
            with urlopen(f"{base_url}/{source['path']}", timeout=30) as response:
                themes = json.load(response)
            validate_catalog(themes)
            with urlopen(f"{base_url}/LICENSE", timeout=30) as response:
                license_text = response.read()

            source.pop("themes", None)
            source["theme_count"] = len(themes)
            source["import_command"] = "python3 scripts/import_themes.py"
            (DESTINATION / "themes.json").write_text(
                json.dumps(themes, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
            )
            (DESTINATION / "LICENSE").write_bytes(license_text)
            (DESTINATION / "source.json").write_text(
                json.dumps(source, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
            )
    except (OSError, ValueError, KeyError) as error:
        parser.error(str(error))
    print(f"Validated {len(themes)} themes at {source['revision']}")


if __name__ == "__main__":
    main()
