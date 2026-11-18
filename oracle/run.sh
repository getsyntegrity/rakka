#!/usr/bin/env bash
# Runs the Pekko 2.0.0 oracle on JDK 17. Usage: oracle/run.sh path-from-string  (JSON string literal on stdin)
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export JAVA_HOME="${JAVA_HOME:-/opt/homebrew/opt/openjdk@17}"
exec "$JAVA_HOME/bin/java" -Xshare:auto -XX:TieredStopAtLevel=1 -cp "$here/target/classes:$here/target/lib/*" pilot.Cli "$@"
