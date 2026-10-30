#!/usr/bin/env bash
# Runs a jshell oracle script against the reference: Apache Pekko compiled from
# upstream/pekko at v1.7.1 (9a54d1294c3ac7821bbed3c940769c3d5027d1ff) via
# tools/oracle/sbt.sh. The classpath is cached in .oracle/source-classpath.txt
# (delete it to rebuild). ORACLE=jars uses the published Maven Central jars
# instead, only as a cross-check.
set -euo pipefail
cd "$(dirname "$0")/../.."
case "${ORACLE:-source}" in
  source)
    cpfile=.oracle/source-classpath.txt
    if [[ ! -s "$cpfile" ]]; then
      tools/oracle/sbt.sh actor/compile "export actor/Runtime/fullClasspath" </dev/null |
        tail -n 1 >"$cpfile.tmp"
      grep -q '^/.*/actor/target/' "$cpfile.tmp" || { echo "sbt build failed" >&2; exit 1; }
      mv "$cpfile.tmp" "$cpfile"
    fi
    cp=$(<"$cpfile") ;;
  jars)
    [[ -d .oracle/jars ]] || tools/oracle/fetch-jars.sh >/dev/null
    cp=$(ls .oracle/jars/*.jar | paste -sd: -) ;;
  *) echo "ORACLE must be 'source' (default) or 'jars'" >&2; exit 2 ;;
esac
exec jshell --class-path "$cp" -q "$@"
