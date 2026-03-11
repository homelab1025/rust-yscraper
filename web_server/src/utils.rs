#[derive(Debug, PartialEq)]
pub enum ExtractIdError {
    NotFound,
    NegativeId(i64),
}

pub fn extract_item_id_from_url(url: &str) -> Result<i64, ExtractIdError> {
    let Some(qs) = url.split('?').nth(1) else {
        return Err(ExtractIdError::NotFound);
    };
    for part in qs.split('&') {
        if let Some(n) = part.strip_prefix("id=").and_then(|v| v.parse::<i64>().ok()) {
            if n < 0 {
                return Err(ExtractIdError::NegativeId(n));
            }
            return Ok(n);
        }
    }
    Err(ExtractIdError::NotFound)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_id_simple() {
        assert_eq!(extract_item_id_from_url("https://example.com/item?id=42"), Ok(42));
    }

    #[test]
    fn test_extract_id_with_multiple_params() {
        assert_eq!(
            extract_item_id_from_url("https://example.com/item?foo=bar&id=99&baz=qux"),
            Ok(99)
        );
    }

    #[test]
    fn test_extract_id_no_query_string() {
        assert_eq!(extract_item_id_from_url("https://example.com/item"), Err(ExtractIdError::NotFound));
    }

    #[test]
    fn test_extract_id_missing_id_param() {
        assert_eq!(extract_item_id_from_url("https://example.com/item?foo=bar"), Err(ExtractIdError::NotFound));
    }

    #[test]
    fn test_extract_id_invalid_value() {
        assert_eq!(extract_item_id_from_url("https://example.com/item?id=notanumber"), Err(ExtractIdError::NotFound));
    }

    #[test]
    fn test_extract_id_negative() {
        assert_eq!(extract_item_id_from_url("https://example.com/item?id=-7"), Err(ExtractIdError::NegativeId(-7)));
    }

    #[test]
    fn test_extract_id_first_param() {
        assert_eq!(extract_item_id_from_url("https://example.com/?id=123&other=456"), Ok(123));
    }
}
