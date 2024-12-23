mod wiki_info;

// Tests
#[cfg(test)]
mod tests {
    use crate::wiki_info::{get_page_similarity, page_from_url, url_utils::resolve_wiki_url};

    use super::wiki_info::{get_most_similar_page, page_from_title, Page};

    #[test]
    fn test_page_from_title() {
        let page = page_from_title("Paris").unwrap();
        assert_eq!(page.title, "Paris");
    }

    #[test]
    fn test_get_most_sim_pages() {
        let main: Page = page_from_title("Paris").  unwrap();

        let pages_to_check: Vec<Page> = vec![
            "France",
            "European Union",
            "World War I",
            "Prime Minister of France",
        ]
        .iter()
        .map(|title| page_from_title(title).unwrap())
        .collect();

        let most_similar_page = get_most_similar_page(&main, &pages_to_check);

        assert_eq!(pages_to_check[most_similar_page].title, "France")
    }

    #[test]
    fn test_same_documents_eq() {
        let page = page_from_title("Prime Minister of France").unwrap();
        let same_page = page_from_title("Prime Minister of France").unwrap();

        let sim_score = get_page_similarity(&page, &same_page);

        assert!(sim_score > 0.98, "Same page should be practically identical");
    }

    #[test]
    fn test_url_resolve() {
        assert_eq!(
            resolve_wiki_url("Prime Minister of France").unwrap(),
            "https://en.wikipedia.org/wiki/Prime_Minister_of_France"
        )
    }


    #[test]
    fn test_doc_difference() {
        let page1 = page_from_url("https://en.wikipedia.org/wiki/Prime_Minister_of_France").unwrap();

        let page2 = page_from_url("https://en.wikipedia.org/wiki/The_Dark_Tower_(series)").unwrap();

        let sim = get_page_similarity(&page1, &page2);
        assert!(sim < 0.2, "Pages should differ signifcantly");
    }
}
