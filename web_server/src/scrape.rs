use crate::CommentRecord;
use crate::scrape::ScrapeError::{ElementSelectorError, HtmlFetchError, InvalidThreadTitle};
use crate::utils::{extract_item_id_from_url, ExtractIdError};
use log::{error, info, warn};
use scraper::error::SelectorErrorKind;
use scraper::{Html, Selector};
use std::error::Error;
use std::fmt::Display;
use std::time::Duration;

#[derive(Debug)]
pub enum ScrapeError {
    HtmlFetchError(reqwest::Error),
    ElementSelectorError(),
    InvalidThreadTitle(),
}

#[derive(Debug)]
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
        }
    }
}

pub(crate) async fn get_comments(url: &str) -> Result<ScrapeResult, ScrapeError> {
    info!("Fetching URL: {}", url);
    let html = match fetch_html(url).await {
        Ok(h) => h,
        Err(e) => {
            return Err(HtmlFetchError(e));
        }
    };

    // Validate thread title for real HN item pages only
    let title = get_thread_title(&html)?;
    if !title
        .to_lowercase()
        .starts_with(THREAD_PREFIX.to_lowercase().as_str())
    {
        warn!("Invalid thread title: {}", title);
        return Err(InvalidThreadTitle());
    }

    let (thread_month, thread_year) = match extract_month_year(&title) {
        Ok((m, y)) => (Some(m), Some(y)),
        Err(e) => {
            error!("Failed to extract month and year from title: {}", e);
            (None, None)
        }
    };

    info!("Parsing root comments...");
    let comments = parse_root_comments(&html);
    match comments {
        Ok(c) => Ok(ScrapeResult {
            comments: c,
            thread_month,
            thread_year,
        }),
        Err(_e) => Err(ElementSelectorError()),
    }
}

async fn fetch_html(url: &str) -> Result<String, reqwest::Error> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .user_agent("rust-yscraper/0.1 (+https://news.ycombinator.com)")
        .build()?;

    let resp = client.get(url).send().await?;

    let valid_response = resp.error_for_status()?;
    valid_response.text().await
}

const THREAD_PREFIX: &str = "Ask HN: What Are You Working On";

fn get_thread_title(html: &str) -> Result<String, ScrapeError> {
    let document = Html::parse_document(html);
    let tl_sel = Selector::parse("span.titleline").map_err(|_| ElementSelectorError())?;
    if let Some(span) = document.select(&tl_sel).next() {
        return Ok(span.text().collect::<String>());
    }
    Err(InvalidThreadTitle())
}

pub(crate) fn extract_month_year(title: &str) -> Result<(i32, i32), String> {
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

fn parse_root_comments(html: &str) -> Result<Vec<CommentRecord>, SelectorErrorKind<'_>> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn test_extract_month_year() {
        assert_eq!(
            extract_month_year("Ask HN: What Are You Working On (February 2026)"),
            Ok((2, 2026))
        );
        assert_eq!(
            extract_month_year("Ask HN: What Are You Working On (january 2025)"),
            Ok((1, 2025))
        );
        assert_eq!(
            extract_month_year("Something else December 2024"),
            Ok((12, 2024))
        );
        assert!(extract_month_year("Ask HN: What Are You Working On").is_err());
        assert!(extract_month_year("Ask HN: What Are You Working On (NotAMonth 2026)").is_err());
    }

    // Use current_thread for faster, deterministic tests
    #[tokio::test(flavor = "current_thread")]
    async fn get_comments_happy_path_parses_one_root_comment() {
        // Arrange: start mock server and stub HTML-resembling HN structure
        let server = MockServer::start().await;
        let html = include_str!("../tests/fixtures/hn_happy_root_and_child.html");

        Mock::given(method("GET"))
            .and(path("/hn"))
            .respond_with(ResponseTemplate::new(200).set_body_string(html))
            .mount(&server)
            .await;

        // Act
        let url = format!("{}/hn", &server.uri());
        let result = get_comments(&url).await;

        // Assert
        assert!(result.is_ok(), "expected Ok, got {:?}", result);
        let scrape_result = result.unwrap();
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
        // Arrange
        let server = MockServer::start().await;
        let html = include_str!("../tests/fixtures/hn_subcomments.html");
        Mock::given(method("GET"))
            .and(path("/sub"))
            .respond_with(ResponseTemplate::new(200).set_body_string(html))
            .mount(&server)
            .await;

        // Act
        let url = format!("{}/sub", &server.uri());
        let result = get_comments(&url).await;

        // Assert
        assert!(result.is_ok());
        let scrape_result = result.unwrap();
        let comments = scrape_result.comments;
        assert_eq!(comments.len(), 2);

        // Alice has n="5"
        let alice = comments.iter().find(|c| c.author == "alice").unwrap();
        assert_eq!(alice.subcomment_count, 5);

        // Bob has no togg/clicky, defaults to 0
        let bob = comments.iter().find(|c| c.author == "bob").unwrap();
        assert_eq!(bob.subcomment_count, 0);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn get_comments_returns_err_on_http_500() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/boom"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&server)
            .await;

        let url = format!("{}/boom", &server.uri());
        let result = get_comments(&url).await;
        assert!(result.is_err(), "expected Err on non-200 status");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn get_comments_returns_err_on_connect_error() {
        // Choose an address likely closed. Port 1 is typically privileged/closed.
        let url = String::from("http://127.0.0.1:1/unreachable");
        let result = get_comments(&url).await;
        assert!(result.is_err(), "expected Err on connect error");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn get_comments_skips_first_with_invalid_id_keeps_second() {
        // Arrange
        let server = MockServer::start().await;
        let html = include_str!("../tests/fixtures/hn_first_invalid_second_ok.html");
        Mock::given(method("GET"))
            .and(path("/mix1"))
            .respond_with(ResponseTemplate::new(200).set_body_string(html))
            .mount(&server)
            .await;

        // Act
        let url = format!("{}/mix1", &server.uri());
        let result = get_comments(&url).await;

        // Assert
        assert!(result.is_ok(), "expected Ok, got {:?}", result);
        let scrape_result = result.unwrap();
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
        // Arrange
        let server = MockServer::start().await;
        let html = include_str!("../tests/fixtures/hn_first_empty_text_second_with_text.html");
        Mock::given(method("GET"))
            .and(path("/mix2"))
            .respond_with(ResponseTemplate::new(200).set_body_string(html))
            .mount(&server)
            .await;

        // Act
        let url = format!("{}/mix2", &server.uri());
        let result = get_comments(&url).await;

        // Assert
        assert!(result.is_ok(), "expected Ok, got {:?}", result);
        let scrape_result = result.unwrap();
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
        // Arrange: HTML has a .comment-like div but lacks required selectors (no tr.athing.comtr)
        let server = MockServer::start().await;
        let html = include_str!("../tests/fixtures/hn_unparsable_with_comment_section.html");
        Mock::given(method("GET"))
            .and(path("/unparsable"))
            .respond_with(ResponseTemplate::new(200).set_body_string(html))
            .mount(&server)
            .await;

        // Act
        let url = format!("{}/unparsable", &server.uri());
        let result = get_comments(&url).await;

        // Assert: parse succeeds (no selector creation error), but no root comments are found
        assert!(result.is_ok(), "expected Ok, got {:?}", result);
        let scrape_result = result.unwrap();
        assert!(
            scrape_result.comments.is_empty(),
            "expected empty vector, got {:?}",
            scrape_result.comments
        );
    }
}
