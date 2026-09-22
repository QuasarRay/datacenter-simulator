#!/usr/bin/env bash
# SPDX-License-Identifier: RPL-1.5
# Verify the complete archive set before allowing Galaxy to extract/install it.
set -euo pipefail
if [[ $# != 4 ]]; then
  echo "usage: $0 ANSIBLE_GALAXY ROLES_DIR COLLECTIONS_DIR CACHE_DIR" >&2
  exit 2
fi
installer=$(readlink -f "$1")
roles=$(realpath -m "$2")
collections=$(realpath -m "$3")
cache=$(realpath -m "$4")
lock=$(dirname "$(readlink -f "$0")")/galaxy.lock.json
jq -e 'all(.[]; (.file | test("^[a-zA-Z0-9_.-]+\\.tar\\.gz$")) and (.name | test("^[a-zA-Z0-9_]+\\.[a-zA-Z0-9_]+$")) and (.sha256 | test("^[a-f0-9]{64}$")) and (.url | startswith("https://")) and (.kind == "role" or .kind == "collection"))' "$lock" >/dev/null
mkdir -p "$cache"
while IFS=$'\t' read -r file url sha; do
  if [[ ! -f "$cache/$file" ]]; then
    curl --fail --location --retry 3 --connect-timeout 15 --max-time 180 "$url" -o "$cache/$file.part"
    mv "$cache/$file.part" "$cache/$file"
  fi
  printf '%s  %s\n' "$sha" "$cache/$file" | sha256sum --check --status
done < <(jq -r '.[] | [.file,.url,.sha256] | @tsv' "$lock")
jq --arg cache "$cache" '{roles:[.[] | select(.kind == "role") | {name:.name,src:($cache+"/"+.file)}]}' "$lock" > "$cache/roles.yml"
"$installer" role install --force --no-deps -r "$cache/roles.yml" -p "$roles"
mapfile -t archives < <(jq -r --arg cache "$cache" '.[] | select(.kind == "collection") | $cache+"/"+.file' "$lock")
"$installer" collection install --force --no-deps -p "$collections" "${archives[@]}"
cp "$lock" "$cache/verified-lock.json"
