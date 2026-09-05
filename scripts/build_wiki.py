#!/usr/bin/env python3
"""Build reproducible GitHub Wiki pages and field tables using only Python 3.10+."""

import argparse
from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[1]
PAGES = {
    "Home": "docs/wiki-home.md",
    "Getting-Started": "docs/getting-started.md",
    "API-Reference": "docs/api-reference.md",
    "Data-Model": "docs/data-model.md",
    "Field-Reference": "docs/field-reference.md",
    "Application-Integration": "docs/integration.md",
    "Error-Handling": "docs/error-handling.md",
    "Card-Generations": "docs/card-generations.md",
    "Troubleshooting": "docs/troubleshooting.md",
    "Security": "docs/security.md",
    "Testing": "docs/testing.md",
    "Architecture": "docs/architecture.md",
    "Migration": "docs/migration.md",
    "Contributing": "CONTRIBUTING.md",
    "Wiki-Setup": "docs/wiki-setup.md",
}


def field_reference():
    source = (ROOT / "src/data.rs").read_text(encoding="utf-8")
    output = [
        "# Field reference\n",
        "Generated from the public Rustdoc comments in `src/data.rs`. Edit those "
        "comments and run `python scripts/build_wiki.py` to refresh this page.\n",
        "`Option` fields can be absent. Consult the containing group's read status "
        "before using them. All accessors and fields refer to the same owned snapshot.\n",
    ]
    for name in ["EmiratesIdData", "NonModifiableData", "ModifiableData", "ReadStatus"]:
        match = re.search(r"pub struct " + name + r" \{(.*?)\n\}", source, re.S)
        if match is None:
            raise ValueError(f"Public model {name} not found")
        output.extend([f"## {name}\n", "| Field | Rust type | Meaning |\n| --- | --- | --- |"])
        comments = []
        for line in match.group(1).splitlines():
            line = line.strip()
            if line.startswith("///"):
                comments.append(line[3:].strip())
            elif line.startswith("pub "):
                field = re.fullmatch(r"pub (\w+): (.*),", line)
                if field is None or not comments:
                    raise ValueError(f"Undocumented or unsupported field: {line}")
                description = " ".join(comments).replace("|", "\\|")
                output.append(f"| `{field[1]}` | `{field[2]}` | {description} |")
                comments = []
        output.append("")
    return "\n".join(output).rstrip() + "\n"


def wiki_links(text, source_path):
    """Rewrite repository-relative page links for GitHub Wiki's extensionless URLs."""
    lookup = {(ROOT / path).resolve(): name for name, path in PAGES.items()}

    def replace(match):
        label, target = match.groups()
        if "://" in target or target.startswith("#"):
            return match.group(0)
        path, separator, anchor = target.partition("#")
        resolved = (source_path.parent / path).resolve()
        if resolved in lookup:
            target = lookup[resolved] + (separator + anchor if separator else "")
        else:
            if not resolved.exists():
                raise ValueError(f"Broken local link in {source_path.name}: {target}")
            relative = resolved.relative_to(ROOT).as_posix()
            target = "https://github.com/k3beidli/emirates-id-reader/blob/main/" + relative
            if separator:
                target += separator + anchor
        return f"[{label}]({target})"

    return re.sub(r"\[([^\]]+)\]\(([^)]+)\)", replace, text)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="Fail if generated files are stale; do not write")
    args = parser.parse_args()
    fields = field_reference()
    outputs = {ROOT / "docs/field-reference.md": fields}
    for name, path in PAGES.items():
        source = ROOT / path
        text = fields if name == "Field-Reference" else source.read_text(encoding="utf-8")
        outputs[ROOT / "docs/wiki" / f"{name}.md"] = wiki_links(text, source).rstrip() + "\n"
    outputs[ROOT / "docs/wiki/_Sidebar.md"] = "\n".join(
        f"- [{name.replace('-', ' ')}]({name})" for name in PAGES
    ) + "\n"
    outputs[ROOT / "docs/wiki/_Footer.md"] = (
        "[Source repository](https://github.com/k3beidli/emirates-id-reader) | "
        "[Getting started](Getting-Started) | [Security](Security)\n\n"
        "Unofficial Rust SDK. Windows contact PC/SC. "
        "Generated from the repository documentation; edit the source guides.\n"
    )
    stale = []
    for path, content in outputs.items():
        current = path.read_text(encoding="utf-8") if path.exists() else None
        if current != content:
            stale.append(path.relative_to(ROOT).as_posix())
            if not args.check:
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(content, encoding="utf-8", newline="\n")
    extra = set((ROOT / "docs/wiki").glob("*.md")) - set(outputs)
    if extra:
        print("Unexpected Wiki pages; review manually: " + ", ".join(p.name for p in sorted(extra)))
        return 1
    if args.check and stale:
        print("Generated documentation is stale:\n" + "\n".join(stale))
        return 1
    print(f"Wiki and field reference {'checked' if args.check else 'generated'} ({len(outputs)} files).")
    return 0


if __name__ == "__main__":
    sys.exit(main())
