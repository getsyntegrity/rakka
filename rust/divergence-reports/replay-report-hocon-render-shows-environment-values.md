# App replay

Build: `/var/folders/2n/mgyt43vn1391pyy5q8wv31kh0000gn/T/pekko-mutants-b87a6ub3/target-hocon-render-shows-environment-values/release/examples/classic-parity`

Passed: 2007/2009

Fallback rate: not reported by this build

## Failing cases

### ch-0184

Arguments: `config-hocon-case`

stdout:

```diff
- RW-02 an environment variable value is not shown -> ok {\u000A    # env variables\u000A    "x" : "<env variable>"\u000A}\u000A
+ RW-02 an environment variable value is not shown -> ok {\u000A    # env variables\u000A    "x" : "/Users/<username>/.opencode/bin:/Users/<username>/Library/Application Support/carapace/bin:/opt/homebrew/bin:/opt/homebrew/sbin:/Users/<username>/.local/bin:/Users/<username>/.opencode/bin:/Users/<username>/.cargo/bin:/Users/<username>/.volta/bin:/Users/<username>/.bun/bin:/Users/<username>/.nix-profile/bin:/nix/var/nix/profiles/default/bin:/usr/local/bin:/Users/<username>/.config:/Users/<username>/.cargo/bin:/usr/local/lib/*:/usr/local/bin:/System/Cryptexes/App/usr/bin:/usr/bin:/bin:/usr/sbin:/sbin:/var/run/com.apple.security.cryptexd/codex.system/bootstrap/usr/local/bin:/var/run/com.apple.security.cryptexd/codex.system/bootstrap/usr/bin:/var/run/com.apple.security.cryptexd/codex.system/bootstrap/usr/appleinternal/bin:/opt/pmk/env/global/bin:/usr/local/go/bin:/opt/homebrew/bin:/Applications/Ghostty.app/Contents/MacOS"\u000A}\u000A
```

### ok-user-info-lone-surrog

Arguments: `path-from-string`

status:

```diff
expected exit code 0, got exit code 2
```

stdout:

```diff
- {"ok":true,"toString":"pekko://sys\ud800@host:2552/user","name":"user","elements":["user"],"uid":0,"toStringWithoutAddress":"/user","toSerializationFormat":"pekko://sys\ud800@host:2552/user","address":{"protocol":"pekko","system":"sys\ud800","host":"host","port":2552,"toString":"pekko://sys\ud800@host:2552"}}
```

stderr:

```diff
+ input outside the API domain: lone UTF-16 surrogate (public inputs are &str; decision Q-X-17)
```
