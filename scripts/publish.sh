#!/usr/bin/env bash
set -euo pipefail
gh auth status
cargo generate-lockfile
cargo fmt --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
# Explicit allowlist prevents local dotfiles or user models from being published.
git add .gitignore Cargo.toml Cargo.lock LICENSE README.md src scripts packaging .github
git diff --cached --check
if ! git diff --cached --quiet; then
  git -c user.name='STL Viewer contributors' -c user.email='contributors@example.invalid' commit -m 'Implement native STL viewer and Linux release automation'
fi
[[ -z "$(git status --porcelain)" ]] || { echo 'Untracked or modified files remain; review before publishing.' >&2; exit 1; }
version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
if ! git remote get-url origin >/dev/null 2>&1; then
  gh repo create stl-viewer-linux --public --source=. --remote=origin
fi
git push -u origin main
sha=$(git rev-parse HEAD)
wait_for_run() {
  local event=$1 run_id=''
  for i in {1..60}; do
    run_id=$(gh run list --workflow build.yml --commit "$sha" --event "$event" --limit 10 --json databaseId,headBranch --jq "[.[] | select(.headBranch == \"$2\")][0].databaseId // empty")
    [[ -z "$run_id" ]] || break
    sleep 2
  done
  [[ -n "$run_id" ]] || { echo 'Workflow did not appear.' >&2; return 1; }
  gh run watch "$run_id" --exit-status
}
wait_for_run push main
if ! git rev-parse "v$version" >/dev/null 2>&1; then git -c user.name='STL Viewer contributors' -c user.email='contributors@example.invalid' tag -a "v$version" -m "Release $version"; fi
git push origin "v$version"
wait_for_run push "v$version"
gh release view "v$version"
