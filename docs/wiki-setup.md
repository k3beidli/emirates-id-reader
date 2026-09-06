<a id="github-wiki-setup"></a>

# Documentation maintenance

The canonical guides live in `docs/`. `scripts/build_wiki.py` turns them into
the Wiki-ready pages in `docs/wiki/`, derives the field reference from the
Rustdoc comments in `src/data.rs` (snapshot fields become accessor rows), and writes the navigation. CI fails if any
generated file is stale, so generate and commit in the same change.

Never edit anything in `docs/wiki/` or `docs/field-reference.md` directly.
The Wiki footer takes the library version from `Cargo.toml`; regenerate it whenever
the package version changes, and update the README, Home, installation guide,
and changelog in the same change.

<a id="regenerate-the-pages"></a>

## Regenerate

With Python 3.10 or newer:

```powershell
python scripts/build_wiki.py
python scripts/build_wiki.py --check
```

## Adding or renaming a page

A page name is its published URL, so **keep the page name stable** even when the
title or the sidebar label changes; that is what `LABELS` is for. To add a page:

1. Write the guide in `docs/`.
2. Add a `PAGES` entry mapping the wiki page name to the source path.
3. Add a `LABELS` entry if the sidebar text differs from the page name, and
   place the page in a `SIDEBAR` group. The generator refuses to run if a page
   is missing from the navigation, listed twice, or unknown.
4. If the guide contains a Rust example, add it to the `guide_examples` block in
   `src/lib.rs`. Examples are only compiled if they are listed there.
5. Update links to any moved sections and retain useful old section anchors.
6. Regenerate and run `cargo test --locked --all-features`. Review the generated
   pages and sidebar, then run `python scripts/build_wiki.py --check`.

Every fenced block needs a language tag. Rustdoc treats an untagged fence as
Rust and will try to compile it, so a bare fence holding shell commands breaks
the build. Card examples use `rust,no_run`; examples that only format values can
execute against synthetic data.

Removing a page also means deleting its generated `docs/wiki/*.md`, since the
generator reports leftover pages, and repointing every inbound link. A link to a
source file that is no longer a page silently becomes a link to GitHub's blob
view rather than a wiki page.

## Writing a guide

Start with the task the reader wants to complete, followed by a short example.
Explain return types, missing values, and important edge cases. Put protocol
background after the practical guidance, and link to shared conventions rather
than repeating them on every page.

Keep examples synthetic and avoid printing names, identifiers, or image bytes.
Cite official references beside the claims they support and record the document
in [sources and acknowledgments](sources.md). Distinguish specification claims,
library behavior, and historical hardware observations.

<a id="copy-and-publish"></a>
<a id="initialize-the-wiki-once"></a>

## Publish

GitHub stores a Wiki in a separate repository with a `.wiki.git` suffix, and
only its default branch is displayed. Create the first page through the GitHub
UI before cloning. See
[GitHub's Wiki documentation](https://docs.github.com/en/communities/documenting-your-project-with-wikis/adding-or-editing-wiki-pages)
and its
[sidebar and footer documentation](https://docs.github.com/en/communities/documenting-your-project-with-wikis/creating-a-footer-or-sidebar-for-your-wiki);
`_Sidebar.md` and `_Footer.md` are recognised by those filenames.

1. Enable Wikis in repository settings if the Wiki tab is missing.
2. Open the Wiki tab and save the first Home page.
3. Clone the Wiki beside your source checkout and copy the generated pages.

Run from the root of the source checkout. Choose a new sibling directory for the
first clone; with an existing clone, use its path and make sure it is clean and
up to date with `git pull --ff-only` first.

```powershell
git clone https://github.com/k3beidli/emirates-id-reader.wiki.git ../emirates-id-reader.wiki
$wikiPath = (Resolve-Path ../emirates-id-reader.wiki).Path
git -C $wikiPath remote -v
git -C $wikiPath status --short
Copy-Item -Path ./docs/wiki/*.md -Destination $wikiPath
git -C $wikiPath diff --stat
git -C $wikiPath diff
git -C $wikiPath add -- '*.md'
git -C $wikiPath commit -m 'Document the Emirates ID Reader Library'
git -C $wikiPath push
```

Review the displayed remote and diff before committing and pushing. Use the
branch `git clone` checks out; do not assume it is named `main` or `master`. The
copy replaces matching filenames and leaves unrelated pages alone, so a page
removed here must be removed on the Wiki separately.

These commands use your existing Git authentication. No token is stored in this
project, and no workflow publishes the Wiki automatically.
