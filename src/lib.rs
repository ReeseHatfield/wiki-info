// library functions are not used outside of test
#![allow(unused)]

pub mod wiki_info;

// Tests
#[cfg(test)]
mod tests {
    use crate::wiki_info::{
        clean_document, cosine_sim, get_page_similarity, page_from_url,
        url_utils::{self, resolve_wiki_url},
    };

    use super::wiki_info::{get_most_similar_page, page_from_title, Page};

    #[test]
    fn test_page_from_title() {
        let page = page_from_title("Paris").unwrap();
        assert_eq!(page.title, "Paris");
    }

    #[test]
    fn test_get_most_sim_pages() {
        let main: Page = page_from_title("Paris").unwrap();

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

        assert!(
            sim_score > 0.98,
            "Same page should be practically identical"
        );
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
        let page1 =
            page_from_url("https://en.wikipedia.org/wiki/Prime_Minister_of_France").unwrap();

        let page2 = page_from_url("https://en.wikipedia.org/wiki/The_Dark_Tower_(series)").unwrap();

        let sim = get_page_similarity(&page1, &page2);
        assert!(sim < 0.2, "Pages should differ signifcantly");
    }

    #[test]
    fn test_traversal() {
        let starting_url = "https://en.wikipedia.org/wiki/The_Dark_Tower_(series)";
        let starting_page = page_from_url(starting_url).unwrap();

        let link_num = 19;
        let target_link = &starting_page.links[link_num].outlink;

        let target_page = page_from_url(target_link).unwrap();

        println!("Title of {}th link: {}", link_num, target_page.title);

        assert!(
            !target_page.title.is_empty(),
            "The target page should have a title."
        );
    }

    #[test]
    fn test_clean_doc() {
        let starting_url = "https://en.wikipedia.org/wiki/The_Dark_Tower_(series)";
        let starting_page = page_from_url(starting_url).unwrap();

        let cleaned = clean_document(&starting_page);

        println!("Content: {:?}", cleaned);

        let words: Vec<&str> = cleaned.content.split_whitespace().collect();

        assert!(
            !words.contains(&"the"),
            "Should not contain common stop word 'the'"
        );
        assert!(
            !words.contains(&"and"),
            "Should not contain common stop word 'and'"
        );
        assert!(
            !words.contains(&"a"),
            "Should not contain common stop word 'a'"
        );
    }

    use scraper::Html;

    #[test]
    fn test_page_from_title_spec_chars() {
        let title = "Rust_(programming_language)";
        let page = page_from_title(title).unwrap();
        assert_eq!(page.title, "Rust (programming language)");
        assert!(!page.content.is_empty());
        assert!(!page.links.is_empty());
    }

    #[test]
    fn test_page_from_url() {
        let url = "https://en.wikipedia.org/wiki/Rust_(programming_language)";
        let page = page_from_url(url).unwrap();
        assert_eq!(page.title, "Rust (programming language)");
        assert!(!page.content.is_empty());
        assert!(!page.links.is_empty());
    }

    #[test]
    fn test_clean_meta_content() {
        let raw_content = "   Some content \nwith \n\nlots of  whitespace. ";
        let cleaned = crate::wiki_info::clean_meta_content(raw_content);
        assert_eq!(cleaned, "Some content with lots of whitespace.");
    }

    #[test]
    fn test_process_content() {
        let html = r#"
        <html>
            <body>
                <div class="mw-content-container">
                    <main id="content">
                        <p>This is a test paragraph with <a href="/wiki/Test_Link">a link</a>.</p>
                    </main>
                </div>
            </body>
        </html>
        "#;
        let document = Html::parse_document(html);
        let selector = scraper::Selector::parse("div.mw-content-container main#content").unwrap();
        let element = document.select(&selector).next().unwrap();

        let page = crate::wiki_info::process_content(element, "Test Page");
        assert_eq!(page.title, "Test Page");
        assert!(page.content.contains("This is a test paragraph"));
        assert_eq!(page.links.len(), 1);
        assert_eq!(page.links[0].title, "a link");
        assert_eq!(
            page.links[0].outlink,
            "https://en.wikipedia.org/wiki/Test_Link"
        );
    }

    #[test]
    fn test_clean_document() {
        let page = Page {
            title: "Test Page".to_string(),
            content: "The quick brown fox jumps over the lazy dog.".to_string(),
            links: vec![],
        };
        let cleaned = clean_document(&page);
        assert!(cleaned.content.contains("quick"));
        assert!(!cleaned.content.contains("the")); // Assuming "the" is a stop word
    }

    // #[test]
    // fn test_page_to_vec() {
    //     let page = Page {
    //         title: "Test Page".to_string(),
    //         content: "the quick brown fox jumps over the lazy dog".to_string(),
    //         links: vec![],
    //     };
    //     let vec = crate::wiki_info::page_to_vec(&page);
    //     assert_eq!(vec.len(), 6); // stopwords: the, over, the
    //     assert!(vec.iter().all(|&x| x > 0.0));
    // }

    #[test]
    fn test_cosine_similarity() {
        let vec1 = vec![1.0, 0.5, 0.0];
        let vec2 = vec![0.5, 1.0, 0.0];
        let sim = cosine_sim(&vec1, &vec2);
        assert!(sim > 0.7);
    }

    #[test]
    fn test_get_page_similarity() {
        let page1 = Page {
            title: "Page 1".to_string(),
            content: "The quick brown fox jumps over the lazy dog.".to_string(),
            links: vec![],
        };
        let page2 = Page {
            title: "Page 2".to_string(),
            content: "The quick brown cat sleeps under the lazy dog.".to_string(),
            links: vec![],
        };
        let similarity = get_page_similarity(&page1, &page2);
        assert!(similarity > 0.5);
    }

    #[test]
    fn test_url_utils_title_from_url() {
        let url = "https://en.wikipedia.org/wiki/Rust_(programming_language)";
        let title = url_utils::title_from_url(url);
        assert_eq!(title, "Rust (programming language) ");
    }

    #[test]
    fn test_url_utils_resolve_wiki_url() {
        let title = "Rust_(programming_language)";
        let resolved_url = url_utils::resolve_wiki_url(title).unwrap();

        println!("RESOLVED URL: {:?}", resolved_url);

        assert!(resolved_url.contains("https://en.wikipedia.org/wiki/"));
    }

    #[test]
    fn test_invalid_url() {
        let invalid_url = "not-a-url";
        let result = url_utils::resolve_wiki_url(invalid_url);
        assert!(result.is_err(), "Expected error for invalid URL");
    }

    #[test]
    fn test_non_200_status() {
        let mock_url = "https://en.wikipedia.org/wiki/NonexistentPage404"; // hope they never add this page lol
        let result = url_utils::resolve_wiki_url(mock_url);
        assert!(result.is_err(), "Expected error for non-200 HTTP status");
        if let Err(e) = result {
            assert!(
                e.to_string().contains("URL returned status"),
                "Unexpected error message: {}",
                e
            );
        }
    }
}
