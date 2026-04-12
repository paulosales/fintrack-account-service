use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginationMeta {
    pub page: u32,
    pub page_size: u32,
    pub total_count: u64,
    pub total_pages: u32,
}

pub fn normalize_page(page: Option<u32>) -> u32 {
    match page {
        Some(value) if value > 0 => value,
        _ => 1,
    }
}

pub fn normalize_page_size(page_size: Option<u32>) -> u32 {
    match page_size {
        Some(10 | 20 | 30 | 50 | 100) => page_size.unwrap(),
        _ => 10,
    }
}

pub fn build_pagination_meta(page: u32, page_size: u32, total_count: u64) -> PaginationMeta {
    let total_pages = if total_count == 0 {
        0
    } else {
        total_count.div_ceil(page_size as u64) as u32
    };

    PaginationMeta {
        page,
        page_size,
        total_count,
        total_pages,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_page() {
        assert_eq!(normalize_page(None), 1);
        assert_eq!(normalize_page(Some(0)), 1);
        assert_eq!(normalize_page(Some(3)), 3);
    }

    #[test]
    fn test_normalize_page_size() {
        assert_eq!(normalize_page_size(None), 10);
        assert_eq!(normalize_page_size(Some(15)), 10);
        assert_eq!(normalize_page_size(Some(50)), 50);
    }

    #[test]
    fn test_build_pagination_meta() {
        let meta = build_pagination_meta(2, 10, 35);

        assert_eq!(meta.page, 2);
        assert_eq!(meta.page_size, 10);
        assert_eq!(meta.total_count, 35);
        assert_eq!(meta.total_pages, 4);
    }
}
