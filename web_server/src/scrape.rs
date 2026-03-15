use crate::CommentRecord;
use crate::scrape::ScrapeError::{
    ElementSelectorError, HtmlFetchError, InvalidThreadTitle, MissingThreadDate,
};
use crate::utils::{ExtractIdError, extract_item_id_from_url};
use async_trait::async_trait;
use log::{info, warn};
use scraper::error::SelectorErrorKind;
use scraper::{Html, Selector};
use std::error::Error;
use std::fmt::Display;
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, PartialEq)]
pub enum ScrapeError {
    HtmlFetchError(String),
    ElementSelectorError(),
    InvalidThreadTitle(),
    MissingThreadDate(String),
}

#[derive(Debug, PartialEq)]
pub struct ScrapeResult {
    pub comments: Vec<CommentRecord>,
    pub thread_month: Option<i32>,
    pub thread_year: Option<i32>,
}

impl Error for ScrapeError {}
impl Display for ScrapeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HtmlFetchError(e) => write!(f, "failed to fetch HTML: {}", e),
            ElementSelectorError() => write!(f, "failed to create HTML selector"),
            InvalidThreadTitle() => write!(
                f,
                "invalid thread title: expected prefix 'Ask HN: What Are You Working On'"
            ),
            MissingThreadDate(title) => {
                write!(f, "thread title is missing a month and year: {}", title)
            }
        }
    }
}

#[async_trait]
pub trait HttpClient: Send + Sync {
    async fn fetch(&self, url: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;
}

pub struct ReqwestHttpClient {
    inner: reqwest::Client,
}

impl ReqwestHttpClient {
    pub fn new() -> Self {
        let inner = reqwest::Client::builder()
            .timeout(Duration::from_secs(20))
            .user_agent("rust-yscraper/0.1 (+https://news.ycombinator.com)")
            .build()
            .expect("failed to build reqwest client");
        ReqwestHttpClient { inner }
    }
}

impl Default for ReqwestHttpClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl HttpClient for ReqwestHttpClient {
    async fn fetch(&self, url: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let resp = self.inner.get(url).send().await?;
        let valid_response = resp.error_for_status()?;
        Ok(valid_response.text().await?)
    }
}

#[async_trait]
pub trait CommentScraper: Send + Sync {
    async fn get_comments(&self, url: &str) -> Result<ScrapeResult, ScrapeError>;
}

pub struct DefaultScraper {
    client: Arc<dyn HttpClient>,
}

impl DefaultScraper {
    pub fn new(client: Arc<dyn HttpClient>) -> Self {
        DefaultScraper { client }
    }
}

#[async_trait]
impl CommentScraper for DefaultScraper {
    async fn get_comments(&self, url: &str) -> Result<ScrapeResult, ScrapeError> {
        info!("Fetching URL: {}", url);
        let html = match self.client.fetch(url).await {
            Ok(h) => h,
            Err(e) => {
                return Err(HtmlFetchError(e.to_string()));
            }
        };

        // Validate thread title for real HN item pages only
        let title = self.get_thread_title(&html)?;
        if !title
            .to_lowercase()
            .starts_with(THREAD_PREFIX.to_lowercase().as_str())
        {
            warn!("Invalid thread title: {}", title);
            return Err(InvalidThreadTitle());
        }

        let (thread_month, thread_year) = match self.extract_month_year(&title) {
            Ok((m, y)) => (Some(m), Some(y)),
            Err(_) => return Err(MissingThreadDate(title)),
        };

        info!("Parsing root comments...");
        let comments = self.parse_root_comments(&html);
        match comments {
            Ok(c) => Ok(ScrapeResult {
                comments: c,
                thread_month,
                thread_year,
            }),
            Err(_e) => Err(ElementSelectorError()),
        }
    }
}

impl DefaultScraper {
    fn get_thread_title(&self, html: &str) -> Result<String, ScrapeError> {
        let document = Html::parse_document(html);
        let tl_sel = Selector::parse("span.titleline").map_err(|_| ElementSelectorError())?;
        if let Some(span) = document.select(&tl_sel).next() {
            return Ok(span.text().collect::<String>());
        }
        Err(InvalidThreadTitle())
    }

    fn parse_root_comments(
        &self,
        html: &str,
    ) -> Result<Vec<CommentRecord>, SelectorErrorKind<'static>> {
        let document = Html::parse_document(html);
        let tr_sel = Selector::parse("tr.athing.comtr")?;
        let ind_img_sel = Selector::parse("td.ind img")?;
        let author_sel = Selector::parse("a.hnuser")?;
        let age_sel = Selector::parse("span.age")?;
        let age_link_sel = Selector::parse("span.age a")?;
        let text_sel = Selector::parse(".comment")?;
        let subcomments_sel = Selector::parse(".togg.clicky")?;

        let mut parsed_comments = Vec::new();

        for tr in document.select(&tr_sel) {
            // Determine indent: root comments have width=0 on the indent img
            let is_root = tr
                .select(&ind_img_sel)
                .next()
                .and_then(|img| img.value().attr("width"))
                .and_then(|w| w.parse::<i32>().ok())
                .map(|w| w == 0)
                .unwrap_or(true); // default to true if missing

            if !is_root {
                continue;
            }

            // Extract comment id from the age link (e.g., <span class="age"><a href="item?id=<COMMENT_ID>">...</a>)
            let id_result = tr
                .select(&age_link_sel)
                .next()
                .and_then(|a| a.value().attr("href"))
                .map(extract_item_id_from_url);

            let id = match id_result {
                Some(Ok(v)) => v,
                Some(Err(ExtractIdError::NegativeId(n))) => {
                    warn!("Skipping a root comment with a negative reply id: {}", n);
                    continue;
                }
                Some(Err(ExtractIdError::NotFound)) | None => {
                    warn!("Skipping a root comment without a parsable reply id");
                    continue;
                }
            };

            let author = tr
                .select(&author_sel)
                .next()
                .map(|a| a.text().collect::<String>())
                .unwrap_or_else(|| "".to_string());

            let date = tr
                .select(&age_sel)
                .next()
                .and_then(|span| span.value().attr("title").map(|s| s.to_string()))
                .or_else(|| tr.select(&age_sel).next().map(|s| s.text().collect()))
                .unwrap_or_else(|| "".to_string());

            let text = tr
                .select(&text_sel)
                .next()
                .map(|t| t.text().collect::<String>())
                .unwrap_or_else(|| "".to_string());

            let subcomment_count = tr
                .select(&subcomments_sel)
                .next()
                .and_then(|a| a.value().attr("n"))
                .and_then(|n| n.parse::<i32>().ok())
                .unwrap_or(0);

            if author.is_empty() && text.trim().is_empty() {
                continue;
            }

            parsed_comments.push(CommentRecord {
                id,
                author,
                date,
                text,
                tags: vec![],
                state: crate::CommentState::New,
                subcomment_count,
            });
        }

        Ok(parsed_comments)
    }

    fn extract_month_year(&self, title: &str) -> Result<(i32, i32), String> {
        use regex::Regex;
        // Regex to match Month (full name) and Year (4 digits), possibly in parentheses
        let re = Regex::new(r"(?i)(January|February|March|April|May|June|July|August|September|October|November|December)\s+(\d{4})").unwrap();

        if let Some(caps) = re.captures(title) {
            let month_str = caps
                .get(1)
                .map(|m| m.as_str().to_lowercase())
                .unwrap_or_default();
            let year_str = caps.get(2).map(|m| m.as_str()).unwrap_or_default();

            let month = match month_str.as_str() {
                "january" => Some(1),
                "february" => Some(2),
                "march" => Some(3),
                "april" => Some(4),
                "may" => Some(5),
                "june" => Some(6),
                "july" => Some(7),
                "august" => Some(8),
                "september" => Some(9),
                "october" => Some(10),
                "november" => Some(11),
                "december" => Some(12),
                _ => None,
            };

            let year = year_str.parse::<i32>().ok();

            if let (Some(m), Some(y)) = (month, year) {
                return Ok((m, y));
            }
        }

        Err(format!(
            "Could not extract month and year from title: '{}'",
            title
        ))
    }
}

const THREAD_PREFIX: &str = "Ask HN: What Are You Working On";

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    struct MockHttpClient(Result<String, &'static str>);

    #[async_trait]
    impl HttpClient for MockHttpClient {
        async fn fetch(
            &self,
            _url: &str,
        ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
            match &self.0 {
                Ok(html) => Ok(html.clone()),
                Err(msg) => Err((*msg).into()),
            }
        }
    }

    fn scraper(html: &str) -> DefaultScraper {
        DefaultScraper::new(Arc::new(MockHttpClient(Ok(html.to_string()))))
    }

    fn scraper_err(msg: &'static str) -> DefaultScraper {
        DefaultScraper::new(Arc::new(MockHttpClient(Err(msg))))
    }

    #[tokio::test(flavor = "current_thread")]
    async fn get_comments_happy_path_parses_one_root_comment() {
        let html = include_str!("../tests/fixtures/hn_happy_root_and_child.html");
        let result = scraper(html).get_comments("http://unused").await;

        assert!(result.is_ok(), "expected Ok, got {:?}", result);
        let scrape_result = result.unwrap();
        assert_eq!(scrape_result.thread_month, Some(10));
        assert_eq!(scrape_result.thread_year, Some(2025));
        let comments = scrape_result.comments;
        assert_eq!(comments.len(), 1, "should only include root comments");
        let c = &comments[0];
        assert_eq!(c.id, 12345);
        assert_eq!(c.author, "alice");
        assert_eq!(c.date, "2025-01-01T12:00:00");
        assert!(c.text.contains("Hello world!"));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn get_comments_extracts_subcomment_count() {
        let html = include_str!("../tests/fixtures/hn_subcomments.html");
        let result = scraper(html).get_comments("http://unused").await;

        assert!(result.is_ok());
        let scrape_result = result.unwrap();
        assert_eq!(scrape_result.thread_month, Some(10));
        assert_eq!(scrape_result.thread_year, Some(2025));
        let comments = scrape_result.comments;
        assert_eq!(comments.len(), 2);

        let alice = comments.iter().find(|c| c.author == "alice").unwrap();
        assert_eq!(alice.subcomment_count, 5);

        let bob = comments.iter().find(|c| c.author == "bob").unwrap();
        assert_eq!(bob.subcomment_count, 0);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn get_comments_returns_err_when_title_has_no_date() {
        let html = r#"<html><body>
            <td class="title"><span class="titleline">Ask HN: What Are You Working On</span></td>
        </body></html>"#;
        let result = scraper(html).get_comments("http://unused").await;
        assert_eq!(
            result,
            Err(ScrapeError::MissingThreadDate(
                "Ask HN: What Are You Working On".to_string()
            ))
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn get_comments_returns_err_on_fetch_error() {
        let result = scraper_err("fetch failed")
            .get_comments("http://unused")
            .await;
        assert!(result.is_err());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn get_comments_skips_first_with_invalid_id_keeps_second() {
        let html = include_str!("../tests/fixtures/hn_first_invalid_second_ok.html");
        let result = scraper(html).get_comments("http://unused").await;

        assert!(result.is_ok(), "expected Ok, got {:?}", result);
        let scrape_result = result.unwrap();
        assert_eq!(scrape_result.thread_month, Some(10));
        assert_eq!(scrape_result.thread_year, Some(2025));
        let comments = scrape_result.comments;
        assert_eq!(
            comments.len(),
            1,
            "only the second valid root comment should remain"
        );
        let c = &comments[0];
        assert_eq!(c.id, 54321);
        assert_eq!(c.author, "dana");
        assert_eq!(c.date, "2025-02-02T08:30:00");
        assert!(c.text.contains("Second valid root comment"));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn get_comments_skips_first_with_empty_author_and_text() {
        let html = include_str!("../tests/fixtures/hn_first_empty_text_second_with_text.html");
        let result = scraper(html).get_comments("http://unused").await;

        assert!(result.is_ok(), "expected Ok, got {:?}", result);
        let scrape_result = result.unwrap();
        assert_eq!(scrape_result.thread_month, Some(10));
        assert_eq!(scrape_result.thread_year, Some(2025));
        let comments = scrape_result.comments;
        assert_eq!(
            comments.len(),
            1,
            "only the second comment with text should remain"
        );
        let c = &comments[0];
        assert_eq!(c.id, 22222);
        assert_eq!(c.author, "eve");
        assert_eq!(c.date, "2025-03-03T10:00:00");
        assert!(c.text.contains("Only this one counts"));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn get_comments_returns_empty_when_page_structure_unparsable_even_with_comment_like_section()
     {
        let html = include_str!("../tests/fixtures/hn_unparsable_with_comment_section.html");
        let result = scraper(html).get_comments("http://unused").await;

        assert!(result.is_ok(), "expected Ok, got {:?}", result);
        let scrape_result = result.unwrap();
        assert_eq!(scrape_result.thread_month, Some(10));
        assert_eq!(scrape_result.thread_year, Some(2025));
        assert!(
            scrape_result.comments.is_empty(),
            "expected empty comments vec"
        );
    }
}
