# Cline / Roo Code Configuration Backup

Backed up: 2026-05-25

## Files

| File | Description |
|------|-------------|
| `vscode_settings.json` | VS Code settings.json — extracted `roo-cline.*` and `claudeCode.*` keys |
| `cline_mcp_settings.json` | MCP server configuration |
| `openrouter_models.json` | OpenRouter model list cache |

## Configuration Summary

### VS Code Settings (`roo-cline.*` keys)
```json
{
  "roo-cline.debug": false,
  "roo-cline.allowedCommands": ["git log", "git diff", "git show"],
  "roo-cline.deniedCommands": [],
  "claudeCode.preferredLocation": "panel"
}
```

### MCP Settings
```json
{
  "mcpServers": {}
}
```

## Restore Instructions

To restore, copy these settings back to:
1. VS Code settings: `~/Library/Application Support/Code/User/settings.json` (merge the `roo-cline.*` keys)
2. MCP settings: `~/Library/Application Support/Code/User/globalStorage/saoudrizwan.claude-dev/settings/cline_mcp_settings.json`
