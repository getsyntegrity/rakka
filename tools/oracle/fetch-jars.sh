#!/usr/bin/env bash
# Downloads the official Apache Pekko 1.7.1 runtime jars used as a behavioral
# oracle into .oracle/jars (git-ignored), verifying each against the SHA-1
# published next to it on Maven Central and against the pinned digests below.
set -euo pipefail
cd "$(dirname "$0")/../.."
dest=.oracle/jars
mkdir -p "$dest"
central=https://repo1.maven.org/maven2
# group path | artifact | version | pinned sha1
jars=(
  "org/apache/pekko|pekko-actor_2.13|1.7.1|6c23c1bd418399622e365ab0f4b53b32b7ca4e5d"
  "org/apache/pekko|pekko-actor-typed_2.13|1.7.1|2a7bd88664707aab6e405e8ca2ad3957ed21cdb9"
  "org/scala-lang|scala-library|2.13.18|eb2086988c25b08705472cc750654570900362e5"
  "com/typesafe|config|1.4.9|fc87584fea1adcf5df899e58501a470329e59b52"
  "org/slf4j|slf4j-api|2.0.17|d9e58ac9c7779ba3bf8142aff6c830617a7fe60f"
)
for entry in "${jars[@]}"; do
  IFS='|' read -r group artifact version pinned <<<"$entry"
  file="$artifact-$version.jar"
  url="$central/$group/$artifact/$version/$file"
  [[ -f "$dest/$file" ]] || curl -fsSL -o "$dest/$file" "$url"
  remote=$(curl -fsSL "$url.sha1" | cut -c1-40)
  local_sha=$(sha1sum "$dest/$file" | cut -c1-40)
  [[ "$remote" == "$local_sha" ]] || { echo "sha1 mismatch for $file" >&2; exit 1; }
  [[ -z "$pinned" || "$pinned" == "$local_sha" ]] || { echo "pinned sha1 mismatch for $file" >&2; exit 1; }
  echo "$local_sha  $file"
done
