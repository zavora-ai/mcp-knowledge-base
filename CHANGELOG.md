# Changelog

## [1.3.0] - 2026-08-13

### Changed
- Upgraded to rmcp 3.1.2 and raised the minimum supported Rust version to 1.94.1.
- Added MCP 2026-07-28 stateless request handling while retaining MCP 2025-11-25 initialization compatibility.

### Added
- Per-request identity and protocol metadata, on-demand discovery/cache hints, and the configured Tasks and sealed MRTR approval policies.

## [1.2.0] - 2025-05-24

### Added
- HealthCheck trait implementation for registry monitoring
- `mcp-server.toml` manifest for ADK registry onboarding
- Structured tracing with `tracing-subscriber` (env-filter)

### Changed
- Edition upgraded to Rust 2024
- Added `adk-mcp-sdk` HealthCheck integration


## [1.1.0] - 2026-05-24

### Added
- Governance: publish_article requires reviewer (high-governance write)
- suggest_article_update creates NEW draft version (never mutates published)
- Actor context on all writes and feedback (actor_id, session_id, ticket_id, reason)
- Audience field: internal, end_user, admin, engineering, security
- Freshness boost (+20% for articles <30 days old)
- Stale penalty (-20% for articles >180 days old)
- Expired/deprecated articles penalized in search
- Not-helpful penalty in ranking
- Feedback linked to ticket_id for traceability
- Search returns compact results with freshness signal
- Architecture SVG diagram

### Changed
- Search only returns published, non-deprecated articles
- Helpfulness boost capped at 30% (prevents popularity bias)

## [1.0.0] - 2026-05-24

### Added
- 9 MCP tools: search, get, list_related, create_draft, publish, suggest_update, feedback, list, gaps
- TF-IDF search with helpfulness boost
- Gap detection (tracks queries with no results)
- Article versioning and draft/publish workflow
- View counting
