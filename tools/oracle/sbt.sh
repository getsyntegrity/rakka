#!/usr/bin/env bash
# Runs sbt on the private Pekko build copy (.oracle/pekko-src, a clone of
# upstream/pekko pinned at v1.7.1 = 9a54d1294c3ac7821bbed3c940769c3d5027d1ff)
# with the JDK 8 toolchain of Pekko's PR-validation workflow
# (.github/workflows/build-test-prValidation.yml): sbt runs on Temurin 8, which
# is also JAVA_8_HOME. (The release toolchain, sbt on JDK 11 + JAVA_8_HOME,
# fails locally in scalac -release 8 on util/Unsafe.java.) sbt, the JDK and every cache live
# under .oracle/ (git-ignored); nothing is written to ~/.sbt, ~/.ivy2,
# ~/.cache/coursier or ~/.m2. Args are forwarded.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
o="$root/.oracle"
rev=9a54d1294c3ac7821bbed3c940769c3d5027d1ff

# fetch <dir> <url> <sha256>: download, verify and unpack into .oracle/<dir>.
fetch() {
  [[ -d "$o/$1" ]] && return
  mkdir -p "$o/$1.tmp"
  curl -fsSL -o "$o/$1.tmp/dist.tgz" "$2"
  echo "$3  $o/$1.tmp/dist.tgz" | sha256sum -c --quiet
  tar xzf "$o/$1.tmp/dist.tgz" -C "$o/$1.tmp"
  mv "$o/$1.tmp" "$o/$1"
}
gh=https://github.com
fetch sbt "$gh/sbt/sbt/releases/download/v1.12.14/sbt-1.12.14.tgz" \
  cd17daae220ff264faa4251334522444518584f0eb2ee82da01523a9b9002b7e
fetch jdk8 "$gh/adoptium/temurin8-binaries/releases/download/jdk8u504-b01/OpenJDK8U-jdk_x64_linux_hotspot_8u504b01.tar.gz" \
  9c70e102f527ac674ac2fe9c7d47b9a04e2d19842ba5ab8e9b33f368bbadfaea
export JAVA_8_HOME="$o/jdk8/jdk8u504-b01"
java="$JAVA_8_HOME/bin/java"
[[ -f "$JAVA_8_HOME/jre/lib/rt.jar" && -x "$java" ]] || { echo "JDK 8 missing under $o" >&2; exit 1; }

if [[ ! -d "$o/pekko-src/.git" ]]; then
  git clone -q --no-hardlinks "$root/upstream/pekko" "$o/pekko-src"
fi
[[ "$(git -C "$o/pekko-src" rev-parse HEAD)" == "$rev" ]] ||
  git -C "$o/pekko-src" -c advice.detachedHead=false checkout -q "$rev"

h="$o/sbt-home"
mkdir -p "$h"
export COURSIER_CACHE="$h/coursier-cache"
export COURSIER_CONFIG_DIR="$h/coursier-config"
unset SBT_OPTS JAVA_OPTS JAVA_TOOL_OPTIONS
# Never pull compiled outputs from Apache's Develocity remote build cache:
# every class must come from compiling these sources locally.
no_remote_cache='set ThisBuild / develocityConfiguration ~= (c => c.withBuildCache(c.buildCache.withRemote(c.buildCache.remote.withEnabled(false))))'
(($#)) || set -- shell
cd "$o/pekko-src"
exec "$java" -Xms1G -Xmx4G -XX:+AlwaysActAsServerClassMachine \
  -Duser.home="$h/home" \
  -Dsbt.global.base="$h/global" \
  -Dsbt.boot.directory="$h/boot" \
  -Dsbt.ivy.home="$h/ivy2" \
  -Dsbt.coursier.home="$h/coursier" \
  -Dsbt.server.autostart=false -Dsbt.supershell=false -Dsbt.color=false -Dsbt.log.noformat=true \
  -jar "$o/sbt/sbt/bin/sbt-launch.jar" "$no_remote_cache" "$@"
