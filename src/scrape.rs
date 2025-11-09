use crate::CommentRecord;
use log::{error, info, warn};
use scraper::{Html, Selector};
use std::time::Duration;

pub async fn get_comments(url: &String) -> Vec<CommentRecord> {
    info!("Fetching URL: {}", url);
    let html = match fetch_html(&url).await {
        Ok(h) => h,
        Err(e) => {
            error!("Failed to fetch '{}': {}", url, e);
            std::process::exit(1);
        }
    };

    info!("Parsing root comments...");
    let comments = parse_root_comments(&html);
    comments
}

async fn fetch_html(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .user_agent("rust-yscraper/0.1 (+https://news.ycombinator.com)")
        .build()?;

    let resp = client.get(url).send().await?;
    // let resp = client.get(url).send()?;
    if !resp.status().is_success() {
        return Err(format!("HTTP error: {}", resp.status()).into());
    }
    let text = resp.text().await?;
    Ok(text)
}

fn parse_root_comments(html: &str) -> Vec<CommentRecord> {
    let document = Html::parse_document(html);
    let tr_sel = Selector::parse("tr.athing.comtr").unwrap();
    let ind_img_sel = Selector::parse("td.ind img").unwrap();
    let author_sel = Selector::parse("a.hnuser").unwrap();
    let age_sel = Selector::parse("span.age").unwrap();
    let text_sel = Selector::parse(".comment").unwrap();
    let reply_sel = Selector::parse("a[href^=\"reply?\"]").unwrap();

    let mut out = Vec::new();

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

        // Extract comment id from the reply link
        let id_opt = tr
            .select(&reply_sel)
            .next()
            .and_then(|a| a.value().attr("href"))
            .and_then(|href| {
                // href format: reply?id=<COMMENT_ID>&...
                href.split('?').nth(1).and_then(|qs| {
                    let mut out_id: Option<i64> = None;
                    for p in qs.split('&') {
                        if let Some(v) = p.strip_prefix("id=") {
                            if let Ok(n) = v.parse::<i64>() {
                                out_id = Some(n);
                                break;
                            }
                        }
                    }
                    out_id
                })
            });

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
            .and_then(|a| Some(a.text().collect::<String>()))
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

        out.push(CommentRecord {
            id,
            author,
            date,
            text,
            tags: vec![],
        });
    }

    out
}
