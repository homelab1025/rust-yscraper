pub fn extract_item_id_from_url(url: &str) -> Option<i64> {
    let qs = url.split('?').nth(1)?;
    for part in qs.split('&') {
        if let Some(n) = part.strip_prefix("id=").and_then(|v| v.parse::<i64>().ok()) {
            return Some(n);
        }
    }
    None
}

// TODO: add tests for the extract function
