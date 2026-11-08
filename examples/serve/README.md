# A team's Surveyor

One `surveyor serve` on a host the team reaches, one dataset that CI keeps current,
every agent's `.mcp.json` pointing at the URL instead of the binary.

```bash
# On the host: the dataset CI writes to, the checkout source is read from.
surveyor --store /srv/surveyor/causeway --repo-id causeway \
  serve --listen 0.0.0.0:7700 --root causeway=/srv/checkouts/causeway \
        --token "$SURVEYOR_TOKEN" --host surveyor.internal

# In CI, after a merge: index what changed into the same dataset.
surveyor --repo-root "$GITHUB_WORKSPACE" --store /srv/surveyor/causeway update
```

The server notices the write within `--reload-every` seconds (30 by default) and
swaps the graph; sessions already open answer from the new one.

`mcp.json` is the Claude Code configuration for that server. The tool names are the
stdio server's, so `examples/claude-code/settings.json` and the `surveyor hook`
bookkeeping work unchanged; only the transport differs. TLS is the reverse proxy's
job: put the port behind the proxy that terminates it and pass the proxy's name with
`--host` so the MCP endpoint accepts it.

The JSON API answers under `/v1` with the structure the CLI's `--json` prints:

```bash
curl -s -H "Authorization: Bearer $SURVEYOR_TOKEN" \
  "http://surveyor.internal:7700/v1/causeway/affected?q=DbError&depth=3" | jq .
```

`docs/serve.md` lists the routes and what was measured.
