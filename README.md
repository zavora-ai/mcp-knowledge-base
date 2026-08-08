# Knowledge Base MCP Server

[![Crates.io](https://img.shields.io/crates/v/mcp-knowledge-base.svg)](https://crates.io/crates/mcp-knowledge-base)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![ADK-Rust Enterprise](https://img.shields.io/badge/ADK--Rust-Enterprise-purple.svg)](https://enterprise.adk-rust.com)
[![Registry Ready](https://img.shields.io/badge/ADK_Registry-Ready-green.svg)](https://www.zavora.ai)

Enterprise knowledge base for [ADK-Rust Enterprise](https://enterprise.adk-rust.com) agents. 9 MCP tools for articles, TF-IDF search boosted by helpfulness, feedback loops, gap detection, versioning, and draft/publish workflow.

## Architecture

<p align="center">
  <img src="https://raw.githubusercontent.com/zavora-ai/mcp-knowledge-base/main/docs/architecture.svg" alt="Knowledge Base MCP Architecture" width="700"/>
</p>

## What It Does

Your agent resolves issues from knowledge before creating tickets. The KB gets smarter over time — helpful articles rank higher, and gaps (queries with no results) tell you what articles to write next.

## Tools (9)

| Tool | What It Does | Use Case |
|------|-------------|----------|
| `search_articles` | TF-IDF search boosted by helpfulness | "Find articles about VPN" |
| `get_article` | Full article with body, stats, version | "Show me KB-001" |
| `list_related_articles` | Related by tags/category | "What else is related?" |
| `create_article_draft` | Create new draft | "Write an article about X" |
| `publish_article` | Make draft searchable | "Publish this article" |
| `suggest_article_update` | Update existing (new version) | "Update this article" |
| `record_article_feedback` | Helpful/not helpful + comment | "This was helpful" |
| `list_articles` | Browse by category/status | "Show all Network articles" |
| `get_article_gaps` | Queries with no results | "What articles are missing?" |

## How It Gets Smarter

1. **Feedback boosts ranking** — articles marked "helpful" score higher in search
2. **Gap detection** — every failed search is tracked. Gaps show what to write next.
3. **View counting** — popular articles surface naturally
4. **Versioning** — every update creates a new version

## Verified Output

```
> search_articles(query: "password reset")
  1 result: "How to reset your password" (score: 8.3, helpful: 1)

> search_articles(query: "printer not working")
  0 results (gap tracked)

> get_article_gaps()
  1 gap: "printer not working" (searched 1x)

> record_article_feedback(article_id: "KB-001", helpful: true)
  recorded: true (boosts future search ranking)
```

## Installation

```bash
cargo install mcp-knowledge-base
```

### MCP client config
```json
{ "mcpServers": { "kb": { "command": "/path/to/mcp-knowledge-base" } } }
```

## Integration with ITSM

The ITSM MCP's `handle_support_request` can use this KB for resolution:
1. User reports issue → ITSM searches KB
2. KB returns relevant article → issue resolved without ticket
3. No result → gap tracked → team writes new article

## Contributors

| [<img src="https://github.com/jkmaina.png" width="80px;"/><br /><sub><b>James Karanja Maina</b></sub>](https://github.com/jkmaina) |
|:---:|

## License

Apache-2.0 — Part of the [ADK-Rust Enterprise](https://enterprise.adk-rust.com) MCP server ecosystem.

## Registry Compliance

This server implements the [ADK MCP SDK](https://crates.io/crates/adk-mcp-sdk) contract:

- **HealthCheck** — async health probe for registry monitoring
- **mcp-server.toml** — manifest declaring tools, risk classes, and credentials
- **Structured tracing** — `RUST_LOG` env-filter for observability

## rmcp and MCP compatibility

This server is built with [`rmcp` 3.1.2](https://github.com/modelcontextprotocol/rust-sdk/releases/tag/rmcp-v3.1.2) and requires Rust 1.88 or newer. The rmcp 3 rollout retains legacy MCP initialization compatibility and targets MCP protocol revisions `2025-11-25` and `2026-07-28`.
