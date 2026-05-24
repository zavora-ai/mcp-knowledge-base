use chrono::{DateTime, Utc};
use rmcp::schemars;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Audience { Internal, EndUser, Admin, Engineering, Security }

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ArticleStatus { Draft, Review, Published, Archived, Deprecated }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
    pub id: String,
    pub title: String,
    pub body: String,
    pub summary: Option<String>,
    pub category: String,
    pub tags: Vec<String>,
    pub audience: Audience,
    pub status: ArticleStatus,
    pub version: u32,
    pub owner: String,
    pub created_by: String,
    pub updated_by: String,
    pub reviewer: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub replaced_by: Option<String>,
    pub helpful_count: u64,
    pub not_helpful_count: u64,
    pub views: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleGap {
    pub query: String,
    pub count: u64,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub suggested_category: Option<String>,
    pub draft_article_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ActorContext {
    pub actor_id: Option<String>,
    pub session_id: Option<String>,
    pub ticket_id: Option<String>,
    pub reason: Option<String>,
}

impl Default for ActorContext {
    fn default() -> Self { Self { actor_id: None, session_id: None, ticket_id: None, reason: None } }
}

pub struct KbStore {
    pub(crate) articles: Mutex<HashMap<String, Article>>,
    feedback: Mutex<Vec<serde_json::Value>>,
    gaps: Mutex<HashMap<String, ArticleGap>>,
}

impl KbStore {
    pub fn new() -> Self {
        Self { articles: Mutex::new(HashMap::new()), feedback: Mutex::new(Vec::new()), gaps: Mutex::new(HashMap::new()) }
    }

    pub fn create_article(&self, title: String, body: String, category: String, tags: Vec<String>, audience: Audience, author: &str) -> Article {
        let now = Utc::now();
        let summary = body.lines().next().map(|l| l.chars().take(120).collect());
        let article = Article {
            id: format!("KB-{}", Uuid::new_v4().simple().to_string()[..8].to_uppercase()),
            title, body, summary, category, tags, audience, status: ArticleStatus::Draft,
            version: 1, owner: author.to_string(), created_by: author.to_string(), updated_by: author.to_string(),
            reviewer: None, created_at: now, updated_at: now, published_at: None,
            expires_at: None, replaced_by: None, helpful_count: 0, not_helpful_count: 0, views: 0,
        };
        self.articles.lock().unwrap().insert(article.id.clone(), article.clone());
        article
    }

    pub fn get_article(&self, id: &str) -> Option<Article> {
        let mut articles = self.articles.lock().unwrap();
        if let Some(a) = articles.get_mut(id) { a.views += 1; Some(a.clone()) } else { None }
    }

    /// suggest_article_update creates a NEW draft version, does not mutate published content
    pub fn suggest_update(&self, source_id: &str, title: Option<String>, body: Option<String>, tags: Option<Vec<String>>, actor: &str) -> Result<Article, String> {
        let source = {
            let articles = self.articles.lock().unwrap();
            articles.get(source_id).cloned().ok_or_else(|| format!("Article not found: {}", source_id))?
        };

        let mut draft = source.clone();

        draft.id = format!("KB-{}", Uuid::new_v4().simple().to_string()[..8].to_uppercase());
        if let Some(t) = title { draft.title = t; }
        if let Some(b) = body { draft.body = b.clone(); draft.summary = Some(b.lines().next().unwrap_or("").chars().take(120).collect()); }
        if let Some(t) = tags { draft.tags = t; }
        draft.status = ArticleStatus::Draft;
        draft.version = source.version + 1;
        draft.updated_by = actor.to_string();
        draft.updated_at = Utc::now();
        draft.published_at = None;

        self.articles.lock().unwrap().insert(draft.id.clone(), draft.clone());
        Ok(draft)
    }

    /// publish requires reviewer
    pub fn publish_article(&self, id: &str, reviewer: &str) -> Result<Article, String> {
        let mut articles = self.articles.lock().unwrap();
        let a = articles.get_mut(id).ok_or_else(|| format!("Article not found: {}", id))?;
        if a.status == ArticleStatus::Published { return Err("Already published".into()); }
        a.status = ArticleStatus::Published;
        a.reviewer = Some(reviewer.to_string());
        a.published_at = Some(Utc::now());
        a.updated_at = Utc::now();
        Ok(a.clone())
    }

    pub fn search(&self, query: &str, category: Option<&str>, audience: Option<&Audience>, limit: usize) -> Vec<(Article, f64)> {
        let lower = query.to_lowercase();
        let terms: Vec<&str> = lower.split_whitespace().filter(|w| w.len() > 2 && !STOP_WORDS.contains(w)).collect();
        if terms.is_empty() { return Vec::new(); }

        let articles = self.articles.lock().unwrap();
        let total = articles.len().max(1) as f64;
        let now = Utc::now();

        let mut scored: Vec<(Article, f64)> = articles.values()
            .filter(|a| a.status == ArticleStatus::Published)
            .filter(|a| category.map_or(true, |c| a.category.to_lowercase() == c.to_lowercase()))
            .filter(|a| audience.map_or(true, |aud| &a.audience == aud))
            .filter(|a| a.replaced_by.is_none()) // exclude deprecated
            .map(|a| {
                let doc = format!("{} {} {} {}", a.title, a.body, a.category, a.tags.join(" ")).to_lowercase();
                let mut score = 0.0;
                for term in &terms {
                    if doc.contains(term) {
                        let df = articles.values().filter(|x| format!("{} {}", x.title, x.body).to_lowercase().contains(term)).count() as f64;
                        score += (total / df.max(1.0)).ln() + 1.0;
                    }
                    if a.title.to_lowercase().contains(term) { score += 3.0; }
                    if a.tags.iter().any(|t| t.to_lowercase().contains(term)) { score += 2.0; }
                }
                // Helpfulness boost (capped)
                if a.helpful_count > 0 { score *= 1.0 + (a.helpful_count as f64 * 0.05).min(0.3); }
                // Freshness boost (articles updated in last 30 days get +20%)
                let days_old = (now - a.updated_at).num_days();
                if days_old < 30 { score *= 1.2; }
                // Stale penalty (>180 days old, -20%)
                if days_old > 180 { score *= 0.8; }
                // Expired penalty
                if a.expires_at.map(|e| now > e).unwrap_or(false) { score *= 0.5; }
                // Not-helpful penalty
                if a.not_helpful_count > a.helpful_count { score *= 0.7; }
                (a.clone(), score)
            })
            .filter(|(_, s)| *s > 0.0)
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(limit);

        if scored.is_empty() {
            let mut gaps = self.gaps.lock().unwrap();
            let gap = gaps.entry(lower.clone()).or_insert(ArticleGap {
                query: query.to_string(), count: 0, first_seen: now, last_seen: now,
                suggested_category: category.map(|s| s.to_string()), draft_article_id: None,
            });
            gap.count += 1;
            gap.last_seen = now;
        }
        scored
    }

    pub fn list_related(&self, article_id: &str, limit: usize) -> Vec<Article> {
        let articles = self.articles.lock().unwrap();
        let source = match articles.get(article_id) { Some(a) => a.clone(), None => return Vec::new() };
        let mut related: Vec<(Article, usize)> = articles.values()
            .filter(|a| a.id != article_id && a.status == ArticleStatus::Published)
            .map(|a| {
                let shared = a.tags.iter().filter(|t| source.tags.contains(t)).count();
                let cat = if a.category == source.category { 2 } else { 0 };
                (a.clone(), shared + cat)
            }).filter(|(_, s)| *s > 0).collect();
        related.sort_by(|a, b| b.1.cmp(&a.1));
        related.truncate(limit);
        related.into_iter().map(|(a, _)| a).collect()
    }

    pub fn record_feedback(&self, article_id: &str, helpful: bool, comment: Option<String>, user: &str, ticket_id: Option<&str>) -> Result<(), String> {
        let mut articles = self.articles.lock().unwrap();
        let a = articles.get_mut(article_id).ok_or_else(|| format!("Article not found: {}", article_id))?;
        if helpful { a.helpful_count += 1; } else { a.not_helpful_count += 1; }
        drop(articles);
        self.feedback.lock().unwrap().push(json!({
            "article_id": article_id, "helpful": helpful, "comment": comment,
            "user": user, "ticket_id": ticket_id, "at": Utc::now().to_rfc3339(),
        }));
        Ok(())
    }

    pub fn get_gaps(&self, limit: usize) -> Vec<ArticleGap> {
        let mut gaps: Vec<ArticleGap> = self.gaps.lock().unwrap().values().cloned().collect();
        gaps.sort_by(|a, b| b.count.cmp(&a.count));
        gaps.truncate(limit);
        gaps
    }

    pub fn list_articles(&self, category: Option<&str>, status: Option<&str>) -> Vec<Article> {
        self.articles.lock().unwrap().values()
            .filter(|a| category.map_or(true, |c| a.category.to_lowercase() == c.to_lowercase()))
            .filter(|a| status.map_or(true, |s| format!("{:?}", a.status).to_lowercase() == s.to_lowercase()))
            .cloned().collect()
    }
}

const STOP_WORDS: &[&str] = &["the","and","for","are","but","not","you","all","can","had","was","one","our","has","have","been","from","with","they","this","that","what","when","how","who","will","just","also","about","need","help","does","into"];
