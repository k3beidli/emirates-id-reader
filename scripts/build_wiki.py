#!/usr/bin/env python3
"""Build reproducible GitHub Wiki pages and field tables using only Python 3.10+."""

import argparse
from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[1]

# Wiki page name -> source guide. The page name is the published URL, so it stays
# stable even when the page's title or sidebar label changes.
PAGES = {
    "Home": "docs/wiki-home.md",
    "Platforms": "docs/platforms.md",
    "Getting-Started": "docs/getting-started.md",
    "Readers-And-Sessions": "docs/readers-and-sessions.md",
    "Data-Model": "docs/data-model.md",
    "Names": "docs/names.md",
    "Codes-And-Identifiers": "docs/codes-and-identifiers.md",
    "Dates": "docs/dates.md",
    "Photos-And-Signatures": "docs/photos-and-signatures.md",
    "Extended-Information": "docs/extended-information.md",
    "Application-Integration": "docs/integration.md",
    "Troubleshooting": "docs/troubleshooting.md",
    "API-Reference": "docs/api-reference.md",
    "Field-Reference": "docs/field-reference.md",
    "Error-Handling": "docs/error-handling.md",
    "Card-Generations": "docs/card-generations.md",
    "Architecture": "docs/architecture.md",
    "Security": "docs/security.md",
    "Contributing": "CONTRIBUTING.md",
    "Testing": "docs/testing.md",
    "Sources": "docs/sources.md",
    "Wiki-Setup": "docs/wiki-setup.md",
}

# Sidebar label, where it differs from the page name. Labels can change freely;
# page names cannot, without breaking published links.
LABELS = {
    "Platforms": "Installation and platforms",
    "Getting-Started": "Your first read",
    "Readers-And-Sessions": "Readers, sessions, and reading options",
    "Data-Model": "Data model and formatting",
    "Codes-And-Identifiers": "Codes and identifiers",
    "Photos-And-Signatures": "Photos and signatures",
    "Extended-Information": "Extended information",
    "Application-Integration": "Application integration",
    "API-Reference": "API reference",
    "Field-Reference": "Field reference",
    "Error-Handling": "Errors and read statuses",
    "Card-Generations": "V1/V2 compatibility",
    "Architecture": "Architecture and chip communication",
    "Security": "Security and access boundaries",
    "Testing": "Testing and hardware validation",
    "Sources": "Sources and acknowledgments",
    "Wiki-Setup": "Documentation maintenance",
}

# Sidebar groups in display order. GitHub Wiki page names share one flat
# namespace, so grouping is presentation only.
SIDEBAR = [
    (None, ["Home"]),
    ("Getting started", ["Platforms", "Getting-Started"]),
    (
        "Using the library",
        [
            "Readers-And-Sessions",
            "Data-Model",
            "Names",
            "Codes-And-Identifiers",
            "Dates",
            "Photos-And-Signatures",
            "Extended-Information",
            "Application-Integration",
            "Troubleshooting",
        ],
    ),
    ("Reference", ["API-Reference", "Field-Reference", "Error-Handling", "Card-Generations"]),
    ("How it works", ["Architecture", "Security"]),
    ("Project", ["Contributing", "Testing", "Sources", "Wiki-Setup"]),
]

MODELS = ["EmiratesIdData", "NonModifiableData", "ModifiableData", "ReadStatus", "ReadOptions"]


def label(name):
    return LABELS.get(name, name.replace("-", " "))


def sidebar():
    """Render the grouped navigation, checking that it covers every page exactly once."""
    placed = [name for _, names in SIDEBAR for name in names]
    duplicated = {name for name in placed if placed.count(name) > 1}
    if duplicated:
        raise ValueError("Pages listed twice in SIDEBAR: " + ", ".join(sorted(duplicated)))
    missing = set(PAGES) - set(placed)
    if missing:
        raise ValueError("Pages missing from SIDEBAR: " + ", ".join(sorted(missing)))
    unknown = set(placed) - set(PAGES)
    if unknown:
        raise ValueError("SIDEBAR names that are not pages: " + ", ".join(sorted(unknown)))
    lines = []
    for group, names in SIDEBAR:
        indent = ""
        if group is not None:
            lines.append(f"- **{group}**")
            indent = "  "
        lines.extend(f"{indent}- [{label(name)}]({name})" for name in names)
    return "\n".join(lines) + "\n"


def struct_table(source, name):
    match = re.search(r"pub struct " + name + r" \{(.*?)\n\}", source, re.S)
    if match is None:
        raise ValueError(f"Public model {name} not found")
    rows = [f"## {name}\n", "| Field or accessor | Rust type | Meaning |\n| --- | --- | --- |"]
    comments = []
    for line in match.group(1).splitlines():
        line = line.strip()
        if line.startswith("///"):
            comments.append(line[3:].strip())
        elif line.startswith(("pub ", "pub(crate) ")):
            field = re.fullmatch(r"pub(?:\(crate\))? (\w+): (.*),", line)
            if field is None or not comments:
                raise ValueError(f"Undocumented or unsupported field: {line}")
            description = " ".join(comments).replace("|", "\\|")
            access = field[1]
            if name == "EmiratesIdData":
                access = {"photo_jpeg": "photo", "holder_signature_image": "signature",
                          "non_modifiable": "identity", "modifiable": "extended"}.get(access, access) + "()"
            field_type = field[2]
            if name == "EmiratesIdData":
                field_type = {"String": "&str", "Option<Vec<u8>>": "Option<&[u8]>",
                              "NonModifiableData": "&NonModifiableData", "ModifiableData": "&ModifiableData",
                              "ReadStatus": "&ReadStatus"}.get(field_type, field_type)
            rows.append(f"| `{access}` | `{field_type}` | {description} |")
            comments = []
    rows.append("")
    return rows


def skipped_fields(source):
    """Derive the not-read section from the constant, so code and docs cannot drift."""
    match = re.search(
        # `[^\n]*` rather than `.*`: DOTALL is needed for the array body only.
        r"((?:^///[^\n]*\n)+)pub const PROTECTED_AND_SKIPPED_FIELDS: &\[&str\] = &\[(.*?)\];",
        source,
        re.S | re.M,
    )
    if match is None:
        raise ValueError("PROTECTED_AND_SKIPPED_FIELDS not found")
    reason = " ".join(line[3:].strip() for line in match[1].splitlines())
    names = re.findall(r'"([^"]+)"', match[2])
    if not names:
        raise ValueError("PROTECTED_AND_SKIPPED_FIELDS is empty")
    return [
        '<a id="fields-not-read-by-this-sdk"></a>\n\n',
        "## Fields not read by this library\n",
        "Not read by this library; access restrictions apply.\n",
        reason + "\n",
        *(f"- {name}" for name in names),
        "",
        "These fields have no getter, snapshot field, or per-group status. "
        "`DataGroupStatus::Protected` instead describes a supported optional group "
        "whose read was refused. See [errors and read statuses](error-handling.md) "
        "and [security and access boundaries](security.md).\n",
    ]


def field_reference():
    source = (ROOT / "src/data.rs").read_text(encoding="utf-8")
    output = [
        "# Field reference\n",
        "Look up the fields in a read result, the status of each group, and the "
        "options used to request it.\n",
        "- [Snapshot](#emiratesiddata)\n"
        "- [Core identity](#nonmodifiabledata)\n"
        "- [Extended information](#modifiabledata)\n"
        "- [Group statuses](#readstatus)\n"
        "- [Read options](#readoptions)\n"
        "- [Fields not read](#fields-not-read-by-this-library)\n",
        "`Option` fields can be absent. Check the containing group's read status "
        "before interpreting `None`. For examples and formatting rules, see "
        "[names](names.md), [codes and identifiers](codes-and-identifiers.md), "
        "[dates](dates.md), [photos and signatures](photos-and-signatures.md), or "
        "[extended information](extended-information.md).\n",
    ]
    for name in MODELS:
        output.extend(struct_table(source, name))
    output.extend(skipped_fields(source))
    output.append(
        "---\n\nThis page is generated from Rustdoc comments in `src/data.rs`. "
        "To change it, edit those comments and run `python scripts/build_wiki.py`; "
        "see [documentation maintenance](wiki-setup.md).\n"
    )
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
    navigation = sidebar()
    manifest = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
    package = manifest.split("[package]", 1)[1].split("\n[", 1)[0]
    version_match = re.search(r'^version\s*=\s*"([^"]+)"', package, re.M)
    if version_match is None:
        raise ValueError("Package version not found in Cargo.toml")
    version = version_match[1]
    outputs = {ROOT / "docs/field-reference.md": fields}
    for name, path in PAGES.items():
        source = ROOT / path
        text = fields if name == "Field-Reference" else source.read_text(encoding="utf-8")
        outputs[ROOT / "docs/wiki" / f"{name}.md"] = wiki_links(text, source).rstrip() + "\n"
    outputs[ROOT / "docs/wiki/_Sidebar.md"] = navigation
    outputs[ROOT / "docs/wiki/_Footer.md"] = (
        "[Source repository](https://github.com/k3beidli/emirates-id-reader) | "
        "[Getting started](Getting-Started) | [Security](Security)\n\n"
        f"Emirates ID Reader Library v{version}. Unofficial. "
        "Windows, Linux, and macOS contact PC/SC. "
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
