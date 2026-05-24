# Changelog

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
