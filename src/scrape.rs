use crate::CommentRecord;
use crate::scrape::ScrapeError::{ElementSelectorError, HtmlFetchError, InvalidThreadTitle};
use crate::utils::extract_item_id_from_url;
use log::{info, warn};
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

pub async fn get_comments(url: &str) -> Result<Vec<CommentRecord>, Box<dyn Error + Send>> {
    info!("Fetching URL: {}", url);
    let html = match fetch_html(url).await {
        Ok(h) => h,
        Err(e) => {
            return Err(Box::new(HtmlFetchError(e)));
        }
    };

    // Validate thread title for real HN item pages only
    if url.starts_with("https://news.ycombinator.com/item")
        && let Err(e) = validate_thread_title(&html)
    {
        return Err(Box::new(e));
    }

    info!("Parsing root comments...");
    let comments = parse_root_comments(&html);
    match comments {
        Ok(c) => Ok(c),
        Err(_e) => Err(Box::new(ElementSelectorError())),
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

fn validate_thread_title(html: &str) -> Result<(), ScrapeError> {
    let document = Html::parse_document(html);
    let tl_sel = Selector::parse("span.titleline").map_err(|_| ElementSelectorError())?;
    if let Some(span) = document.select(&tl_sel).next() {
        let title_text = span.text().collect::<String>();
        if title_text
            .to_lowercase()
            .starts_with(THREAD_PREFIX.to_lowercase().as_str())
        {
            return Ok(());
        } else {
            warn!("Invalid thread title: {}", title_text);
        }
    }
    Err(InvalidThreadTitle())
}

fn parse_root_comments(html: &str) -> Result<Vec<CommentRecord>, SelectorErrorKind<'_>> {
    let document = Html::parse_document(html);
    let tr_sel = Selector::parse("tr.athing.comtr")?;
    let ind_img_sel = Selector::parse("td.ind img")?;
    let author_sel = Selector::parse("a.hnuser")?;
    let age_sel = Selector::parse("span.age")?;
    let age_link_sel = Selector::parse("span.age a")?;
    let text_sel = Selector::parse(".comment")?;

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
        let id_opt = tr
            .select(&age_link_sel)
            .next()
            .and_then(|a| a.value().attr("href"))
            .and_then(extract_item_id_from_url);

        let id = match id_opt {
            Some(v) => v,
            None => {
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

        if author.is_empty() && text.trim().is_empty() {
            continue;
        }

        parsed_comments.push(CommentRecord {
            id,
            author,
            date,
            text,
            tags: vec![],
        });
    }

    Ok(parsed_comments)
}

#[cfg(test)]
mod tests {
    use super::get_comments;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

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
        let comments = result.unwrap();
        assert_eq!(comments.len(), 1, "should only include root comments");
        let c = &comments[0];
        assert_eq!(c.id, 12345);
        assert_eq!(c.author, "alice");
        assert_eq!(c.date, "2025-01-01T12:00:00");
        assert!(c.text.contains("Hello world!"));
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
        let comments = result.unwrap();
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
        let comments = result.unwrap();
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
        let comments = result.unwrap();
        assert!(
            comments.is_empty(),
            "expected empty vector, got {:?}",
            comments
        );
    }
}
