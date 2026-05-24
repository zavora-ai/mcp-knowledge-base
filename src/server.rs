use adk_mcp_sdk::{HealthCheck, HealthStatus};
use crate::store::{ActorContext, Audience, KbStore};
use rmcp::{handler::server::wrapper::Parameters, schemars, tool, tool_router};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SearchArticlesInput { pub query: String, pub category: Option<String>, pub audience: Option<Audience>, pub limit: Option<usize> }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetArticleInput { pub id: String }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListRelatedInput { pub article_id: String, pub limit: Option<usize> }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateArticleDraftInput { pub title: String, pub body: String, pub category: String, #[serde(default)] pub tags: Vec<String>, pub audience: Option<Audience>, pub author: String, #[serde(default)] pub actor: ActorContext }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SuggestUpdateInput { pub source_article_id: String, pub title: Option<String>, pub body: Option<String>, pub tags: Option<Vec<String>>, pub author: String, #[serde(default)] pub actor: ActorContext }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PublishArticleInput { pub id: String, /// Reviewer who approves publication
    pub reviewer: String, #[serde(default)] pub actor: ActorContext }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RecordFeedbackInput { pub article_id: String, pub helpful: bool, pub comment: Option<String>, pub user: String, pub ticket_id: Option<String> }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListArticlesInput { pub category: Option<String>, pub status: Option<String> }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetGapsInput { pub limit: Option<usize> }

#[derive(Clone)]
pub struct KbServer { pub store: Arc<KbStore> }

#[tool_router(server_handler)]
impl KbServer {
    #[tool(description = "Search knowledge base with TF-IDF scoring. Boosted by helpfulness, penalized for staleness. Filters by category and audience. Tracks gaps on no results.")]
    fn search_articles(&self, Parameters(i): Parameters<SearchArticlesInput>) -> String {
        let results = self.store.search(&i.query, i.category.as_deref(), i.audience.as_ref(), i.limit.unwrap_or(10));
        let articles: Vec<serde_json::Value> = results.iter().map(|(a, score)| serde_json::json!({
            "id": a.id, "title": a.title, "summary": a.summary, "category": a.category,
            "tags": a.tags, "audience": a.audience, "score": (score * 10.0).round() / 10.0,
            "helpful": a.helpful_count, "views": a.views, "version": a.version,
            "days_since_update": (chrono::Utc::now() - a.updated_at).num_days(),
        })).collect();
        serde_json::to_string_pretty(&serde_json::json!({"query": i.query, "count": articles.len(), "results": articles})).unwrap()
    }

    #[tool(description = "Get full article by ID — body, version, owner, reviewer, freshness, feedback stats. Increments view count.")]
    fn get_article(&self, Parameters(i): Parameters<GetArticleInput>) -> String {
        match self.store.get_article(&i.id) {
            Some(a) => serde_json::to_string_pretty(&serde_json::json!({
                "id": a.id, "title": a.title, "body": a.body, "summary": a.summary,
                "category": a.category, "tags": a.tags, "audience": a.audience,
                "status": a.status, "version": a.version, "owner": a.owner,
                "reviewer": a.reviewer, "views": a.views,
                "helpful": a.helpful_count, "not_helpful": a.not_helpful_count,
                "created_at": a.created_at, "updated_at": a.updated_at, "published_at": a.published_at,
                "expires_at": a.expires_at, "replaced_by": a.replaced_by,
            })).unwrap(),
            None => format!("Article not found: {}", i.id),
        }
    }

    #[tool(description = "Find articles related to a given article by shared tags and category.")]
    fn list_related_articles(&self, Parameters(i): Parameters<ListRelatedInput>) -> String {
        let related = self.store.list_related(&i.article_id, i.limit.unwrap_or(5));
        let results: Vec<serde_json::Value> = related.iter().map(|a| serde_json::json!({"id": a.id, "title": a.title, "category": a.category, "tags": a.tags})).collect();
        serde_json::to_string_pretty(&serde_json::json!({"article_id": i.article_id, "count": results.len(), "related": results})).unwrap()
    }

    #[tool(description = "Create a new article draft. Requires author. Must be published separately (requires reviewer approval).")]
    fn create_article_draft(&self, Parameters(i): Parameters<CreateArticleDraftInput>) -> String {
        let audience = i.audience.unwrap_or(Audience::Internal);
        let article = self.store.create_article(i.title, i.body, i.category, i.tags, audience, &i.author);
        serde_json::to_string_pretty(&serde_json::json!({"id": article.id, "title": article.title, "status": article.status, "version": article.version, "message": "Draft created. Use publish_article with a reviewer to make it searchable."})).unwrap()
    }

    #[tool(description = "Suggest an update to an existing article. Creates a NEW draft version — does not mutate published content. Requires author.")]
    fn suggest_article_update(&self, Parameters(i): Parameters<SuggestUpdateInput>) -> String {
        match self.store.suggest_update(&i.source_article_id, i.title, i.body, i.tags, &i.author) {
            Ok(draft) => serde_json::to_string_pretty(&serde_json::json!({"draft_id": draft.id, "source_id": i.source_article_id, "version": draft.version, "status": draft.status, "message": "New draft version created. Original article unchanged. Publish when ready."})).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Publish a draft article. HIGH GOVERNANCE: requires reviewer name. Makes article searchable and authoritative.")]
    fn publish_article(&self, Parameters(i): Parameters<PublishArticleInput>) -> String {
        if i.reviewer.is_empty() {
            return serde_json::json!({"error": "Reviewer required. Publishing makes content authoritative — a reviewer must approve."}).to_string();
        }
        match self.store.publish_article(&i.id, &i.reviewer) {
            Ok(a) => serde_json::to_string_pretty(&serde_json::json!({"id": a.id, "title": a.title, "status": a.status, "reviewer": a.reviewer, "published_at": a.published_at})).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Record feedback on an article (helpful/not helpful). Affects search ranking. Include ticket_id for traceability.")]
    fn record_article_feedback(&self, Parameters(i): Parameters<RecordFeedbackInput>) -> String {
        match self.store.record_feedback(&i.article_id, i.helpful, i.comment, &i.user, i.ticket_id.as_deref()) {
            Ok(()) => serde_json::to_string_pretty(&serde_json::json!({"recorded": true, "article_id": i.article_id, "helpful": i.helpful})).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "List articles filtered by category and/or status (draft/published/archived/deprecated).")]
    fn list_articles(&self, Parameters(i): Parameters<ListArticlesInput>) -> String {
        let articles = self.store.list_articles(i.category.as_deref(), i.status.as_deref());
        let results: Vec<serde_json::Value> = articles.iter().map(|a| serde_json::json!({"id": a.id, "title": a.title, "category": a.category, "status": a.status, "audience": a.audience, "version": a.version, "views": a.views, "helpful": a.helpful_count})).collect();
        serde_json::to_string_pretty(&serde_json::json!({"count": results.len(), "articles": results})).unwrap()
    }

    #[tool(description = "Get knowledge gaps — queries with no results, ranked by frequency. Shows what articles need to be written.")]
    fn get_article_gaps(&self, Parameters(i): Parameters<GetGapsInput>) -> String {
        let gaps = self.store.get_gaps(i.limit.unwrap_or(10));
        serde_json::to_string_pretty(&serde_json::json!({"count": gaps.len(), "gaps": gaps})).unwrap()
    }
}

#[async_trait::async_trait]
impl HealthCheck for KbServer {
    async fn check_health(&self) -> HealthStatus {
        HealthStatus {
            healthy: true,
            message: Some("operational".into()),
            latency_ms: Some(1),
        }
    }
}
