# tools/context

`check.sh` keeps `STATUS.md` (the project's state) current across sessions and restarts. The hooks of
`.claude/settings.json` call it:

| Mode | Hook | What it does |
|---|---|---|
| `session` | `SessionStart` | lists the work files (code, dialects, specs, tools) newer than `STATUS.md`: work of another session that was not recorded |
| `stop` | `Stop` | if the session changed work files after the last edit of `STATUS.md`, it does not let the turn end (once per turn) and asks to update it |

The session is marked with a file in `$XDG_RUNTIME_DIR` (`nuclear-context-<session_id>`), so the `stop` mode only
looks at what changed since it started. It does not look at `docs/` nor the context `.md` files: changing only
documentation does not require a Log line.

Testing it by hand:

```sh
echo '{"session_id":"t"}' | tools/context/check.sh session
echo '{"session_id":"t"}' | tools/context/check.sh stop; echo $?   # 2 = STATUS.md needs an update
```
