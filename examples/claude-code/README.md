# Claude Code hooks for Surveyor

`settings.json` wires the four hooks `surveyor hook claude-code` answers. Copy its
`hooks` block into the repository's `.claude/settings.json` (shared with every
cloner) or your own `.claude/settings.local.json`. The adapter takes the repository
root from the payload's `cwd` and the dataset from `--store`, `SURVEYOR_STORE` or
`<root>/.surveyor`; a dataset kept outside the checkout, as Causeway's is, needs the
variables on the command line, for example:

```json
{ "type": "command", "command": "SURVEYOR_STORE='/path/to/store' SURVEYOR_PROFILE='/path/to/causeway.toml' surveyor hook claude-code" }
```

What each event does is in [docs/harness.md](../../docs/harness.md).
