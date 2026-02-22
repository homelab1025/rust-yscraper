pub fn extract_item_id_from_url(url: &str) -> Option<i64> {
    let qs = url.split('?').nth(1)?;
    for part in qs.split('&') {
        if let Some(v) = part.strip_prefix("id=") {
            if let Ok(n) = v.parse::<i64>() {
                return Some(n);
            }
        }
    }
    None
}

// TODO: add tests for the extract function