pub fn extract_item_id_from_url(url: &str) -> Option<i64> {
    let qs = url.split('?').nth(1)?;
    for part in qs.split('&') {
        if let Some(n) = part.strip_prefix("id=").and_then(|v| v.parse::<i64>().ok()) {
            // TODO: return an error for negative id
            if n >= 0 {
                return Some(n);
            }
        }
    }
    None
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_id_simple() {
        assert_eq!(extract_item_id_from_url("https://example.com/item?id=42"), Some(42));
    }

    #[test]
    fn test_extract_id_with_multiple_params() {
        assert_eq!(
            extract_item_id_from_url("https://example.com/item?foo=bar&id=99&baz=qux"),
            Some(99)
        );
    }

    #[test]
    fn test_extract_id_no_query_string() {
        assert_eq!(extract_item_id_from_url("https://example.com/item"), None);
    }

    #[test]
    fn test_extract_id_missing_id_param() {
        assert_eq!(extract_item_id_from_url("https://example.com/item?foo=bar"), None);
    }

    #[test]
    fn test_extract_id_invalid_value() {
        assert_eq!(extract_item_id_from_url("https://example.com/item?id=notanumber"), None);
    }

    #[test]
    fn test_extract_id_negative() {
        assert_eq!(extract_item_id_from_url("https://example.com/item?id=-7"), None);
    }

    #[test]
    fn test_extract_id_first_param() {
        assert_eq!(extract_item_id_from_url("https://example.com/?id=123&other=456"), Some(123));
    }
}
