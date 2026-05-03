# zed-portfolio-presence

A [Zed](https://zed.dev) extension that streams my live coding activity to a portfolio API endpoint using Language Server Protocol. Built as a fork of [xhyrom/zed-discord-presence](https://github.com/xhyrom/zed-discord-presence), replacing the Discord IPC layer with plain HTTP

Every time I open, change, or save a file in Zed, the extension sends a `PUT` request to configured endpoint with a JSON payload describing what I'm working on. Activity is throttled to at most once every 5 seconds and deduplicated

## Installation

This extension is not published to the Zed extension marketplace. Install it as a dev extension:

1. Clone this repository
2. Open Zed and run `zed: install dev extension` via command palette
3. Select the directory where you cloned the repository

## Configuration

Add the following to your Zed `settings.json`

```jsonc
{
    "lsp": {
        "portfolio-presence": {
            "initialization_options": {
                // Required: your API endpoint
                "endpoint_url": "https://example.com/api/activity",

                // Optional: sent as Authorization: Bearer <secret>
                "http_secret": "your-secret-token",

                // Include git remote URL in the payload
                "git_integration": true,

                // Disable presence in specific workspaces
                "rules": {
                    "mode": "blacklist", // or "whitelist"
                    "paths": ["/absolute/path/to/workspace"],
                },
            },
        },
    },
}
```

## Payload

Each `PUT` request sends a JSON body of this shape:

```jsonc
{
    "workspace": {
        "name": "my-project",
        "files": 0,
    },
    "file": {
        "name": "main.rs",
        "language": "rust",
        "line": 42,
    },
    "git": {
        "branch": "main",
        "remote": "https://github.com/username/my-project", // null if git integration disabled
    },
    "timestamp": 1746230400000,
}
```

The `Authorization: Bearer <secret>` header is added when `http_secret` is set

## Building

```sh
cd lsp && cargo build --release
```

The release binary is at `target/release/portfolio-presence-lsp`
