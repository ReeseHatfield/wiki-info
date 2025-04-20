

# **Wiki-Info**

<p align="center">
  <a href="https://github.com/reesehatfield/wiki-info">
    <img src="https://upload.wikimedia.org/wikipedia/en/thumb/8/80/Wikipedia-logo-v2.svg/330px-Wikipedia-logo-v2.svg.png" alt="Wikipedia Logo" width="400" height="370">
  </a>
</p>

<h3 align="center"><strong>Wiki-Info</strong></h3>

<p align="center">
    A high speed, blocking, wikipedia information retrieval API for rust.
  <br>
</p>


## Features

- Fetch Wikipedia pages by title or URL.
- Extract page content and hyperlinks.
- Lightweight and efficient design.
- Comprehensive error handling ecosystem.

This library was primary built for dynamic, wikipedia graph traversal. Each node is only computed on
an as-requested basis. As a result, nodes do not contain backlinks, only outlinks, and similarity metrics are
solely computed based on their term-frequency in the given document and do not contain IDF information.
This could theoretically be added, but would require the traversal of all of wikipedia.

**Please ensure you comply with wikipedia terms of service when using this API** 

## Installation:

Simply run `cargo add wiki-info` to your rust project. See [crate page](https://crates.io/crates/wiki-info) for more details

## Considerations:

If you use this crate, please consider [donating to the Wikimedia Foundation](https://donate.wikimedia.org/wiki/Ways_to_Give)

## Examples:

Getting a Page from the title

```rust
use wiki_info::wiki_info::{page_from_title, WikiError};

fn main() -> Result<(), WikiError> {
    let page = page_from_title("Paris")?;
    println!("Title: {}", page.title);
    println!("Content: {}", page.content);
    println!("Links: {:?}", page.links);
    Ok(())
}
```

Getting a Page from the URL

```rust
use wiki_info::wiki_info::{page_from_url, WikiError};

fn main() -> Result<(), WikiError> {
    let page = page_from_url("https://en.wikipedia.org/wiki/Rust_(programming_language)")?;
    println!("Title: {}", page.title);
    println!("Content: {}", page.content);
    println!("Links: {:?}", page.links);
    Ok(())
}
```

Comparing page similarity

```rust
use wiki_info::wiki_info::{page_from_title, get_page_similarity, WikiError};

fn main() -> Result<(), WikiError> {
    let page1 = page_from_title("Rust_(programming_language)")?;
    let page2 = page_from_title("C++")?;

    let similarity_score = get_page_similarity(&page1, &page2);

    println!("Similarity between '{}' and '{}' is: {:.2}%", page1.title, page2.title, similarity_score * 100.0);
    Ok(())
}
```

Comparing similarity of multiple pages

```rust
use wiki_info::wiki_info::{get_most_similar_page, page_from_title, Page, WikiError};

fn main() -> Result<(), WikiError> {
    // Fetch the base page you want to compare.
    let base_page = page_from_title("Rust (programming language)")?;

    // Titles of pages to compare against.
    let compare_titles = vec![
        "C (programming language)",
        "Go (programming language)",
        "Python (programming language)",
    ];

    // Fetch pages to compare against.
    let compare_pages = compare_titles
        .iter()
        .map(|title| page_from_title(title).unwrap())
        .collect::<Vec<Page>>();

    // Find the most similar page.
    let most_similar = get_most_similar_page(&base_page, &compare_pages);

    println!("Most similar page to rust is {:?}", compare_titles[most_similar]); // -> C

    Ok(())
}
```
