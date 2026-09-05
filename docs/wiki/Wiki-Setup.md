# GitHub Wiki setup

The repository contains Wiki-ready Markdown in `docs/wiki/`. These files are
prepared for publication; pushing the main source repository does not publish
them to the Wiki automatically.

GitHub stores a Wiki in a separate Git repository, with a `.wiki.git` suffix.
Create an initial page through GitHub before cloning it. Only the Wiki's
default branch is displayed. See [GitHub's Wiki editing documentation](https://docs.github.com/en/communities/documenting-your-project-with-wikis/adding-or-editing-wiki-pages).

## Regenerate the pages

From this source checkout, with Python 3.10 or newer:

```powershell
python scripts/build_wiki.py
python scripts/build_wiki.py --check
```

The generator converts guide links to Wiki page links, derives the complete
field reference from Rustdoc comments, and creates `Home.md`, `_Sidebar.md`,
and `_Footer.md`. GitHub recognizes the last two filenames as custom
navigation. See [GitHub's sidebar/footer documentation](https://docs.github.com/en/communities/documenting-your-project-with-wikis/creating-a-footer-or-sidebar-for-your-wiki).

Commit the generated files alongside their source guides. CI checks for
staleness. Edit `docs/*.md`, `CONTRIBUTING.md`, or the public field comments in
`src/data.rs`; do not edit generated pages directly.

## Initialize the Wiki once

1. Open the repository on GitHub and enable Wikis in repository settings if
   the Wiki tab is missing and your repository plan supports it.
2. Open the Wiki tab and create/save the first Home page.
3. Clone the Wiki beside your source checkout using the commands below.

## Copy and publish

Run from the root of the source checkout. Choose a new sibling directory for
the first clone; if you already have a Wiki clone, use its path instead and
first ensure it is clean and updated with `git pull --ff-only`.

```powershell
git clone https://github.com/k3beidli/emirates-id-reader.wiki.git ../emirates-id-reader.wiki
$wikiPath = (Resolve-Path ../emirates-id-reader.wiki).Path
git -C $wikiPath remote -v
git -C $wikiPath status --short
Copy-Item -Path ./docs/wiki/*.md -Destination $wikiPath
git -C $wikiPath diff --stat
git -C $wikiPath diff
git -C $wikiPath add -- '*.md'
git -C $wikiPath commit -m 'Document the Emirates ID Reader SDK'
git -C $wikiPath push
```

Review the displayed remote and diff before committing/pushing. Use the
default branch checked out by `git clone`; do not assume it is named `main`
or `master`. The copy replaces matching page filenames and preserves unrelated
pages. If a source page was renamed/removed, review and remove the old Wiki
page separately so it does not remain published.

These commands use your existing Git authentication. No token needs to be
stored in this project. No Wiki sync workflow or automatic publication is
enabled by this setup.
