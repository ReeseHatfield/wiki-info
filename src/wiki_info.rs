use std::{collections::HashMap, fmt::format};

use scraper::{Html, Selector};

mod stop_words;

/// Singleton module for networking clients. 
/// This is a *blocking* library, should never have race condition on networking side 
mod client {
    use lazy_static::lazy_static;
    use log::debug;
    use reqwest::blocking::Client;
    use std::sync::{Arc, Mutex};

    struct ClientSingleton {
        blocking_client: Arc<reqwest::blocking::Client>,
    }

    impl ClientSingleton {
        fn new() -> Self {
            debug!("Initializing ClientSingleton...");
            ClientSingleton {
                blocking_client: Arc::new(Client::new()),
            }
        }

        // private version of this fn
        fn get_client(&self) -> Arc<Client> {
            debug!("Retrieving client from ClientSingleton...");
            Arc::clone(&self.blocking_client)
        }
    }

    // lazy static init of singleton
    lazy_static! {
        static ref CLIENT_INSTANCE: Mutex<ClientSingleton> = {
            debug!("Creating CLIENT_INSTANCE...");
            std::sync::Mutex::new(ClientSingleton::new())
        };
    }

    /// Get a singleton client instance
    pub fn get_client() -> Arc<Client> {
        debug!("Acquiring lock on CLIENT_INSTANCE...");
        CLIENT_INSTANCE
            .lock()
            .expect("Failed to acquire lock on ClientSingleton") // ehhhh might wanna fix this
            .get_client()
    }
}

use log::debug;

/// Enum of all wiki possible wiki error types.
/// See impl of Display and Error
#[derive(Debug)]
pub enum WikiError {
    NetworkingError(String),
    ParseError(String),
    URLError(String),
}

impl std::error::Error for WikiError {}

impl std::fmt::Display for WikiError {
    /// Standard format display for wiki errors
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NetworkingError(msg) => write!(f, "Networking error: {}", msg),
            Self::ParseError(msg) => write!(f, "Parse error: {}", msg),
            Self::URLError(msg) => write!(f, "URL error {}", msg),
        }
    }
}

/// Gets a Page from a title &str
///
/// # Arguments
/// * `title` - The title of the page
///
/// # Returns
///
/// Ok(Page) - the new wiki page struct
/// Err(WikiError) - error if wiki parsing/fetching fails
pub fn page_from_title(title: &str) -> Result<Page, WikiError> {
    debug!("parse_parse_from_title called...");

    let url = url_utils::resolve_wiki_url(title)?;

    return page_from_url(&url);
}

/// Gets a Page from a url
///
/// # Arguments
///
/// * `url` the url of the wiki page
///
/// # Returns
///
/// Ok(Page) - the new wiki page struct
/// Err(WikiError) - error if wiki parsing/fetching fails
pub fn page_from_url(url: &str) -> Result<Page, WikiError> {
    debug!("parse_page_from_url called with url: {}", url);
    let client = client::get_client();

    debug!("Sending request to URL: {}", url);
    let response = client
        .get(url)
        // lie about user agents lol
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36",
        )
        .send()
        .map_err(|err| WikiError::NetworkingError( // into wiki error
            format!("Request error with status {:?}",  err.status()))
        )?;

    debug!("Response received from URL: {}", url);
    let html_content = handle_response(response)?;

    debug!("Parsing HTML content...");
    let document = Html::parse_document(&html_content);

    // this wierd selector is what gets the actual body from a page
    let selector = Selector::parse("div.mw-content-container main#content").unwrap(); // TODO FIX UNWRAP

    match document.select(&selector).next() {
        Some(content) => {
            debug!("Content successfully selected. Processing content...");

            let title = url_utils::title_from_url(url);
            // process starting at root elem
            Ok(process_content(content, &title))
        }
        None => {
            debug!("Failed to select content from document.");
            Err(WikiError::ParseError(
                "Failed to select content from document.".to_owned(),
            ))
        }
    }
}

/// A URL utility module, primarily for extract and encoding wiki data from urls
pub mod url_utils {
    use reqwest::{blocking::Client, header::LOCATION};
    use std::sync::Arc;

    use super::{client::get_client, WikiError};

    /// Extract a title slug from a url &srt
    ///
    /// # Arguments
    /// * `url` - the url to pull the title from
    ///
    /// # Returns
    /// owned string for the new title
    pub fn title_from_url(url: &str) -> String {
        let title = extract_slug(url)
            .split("_")
            .fold(String::new(), |a, b| a + b + " ");

        return title;
    }

    // util for title extraction
    fn extract_slug(url: &str) -> &str {
        // last elem
        match url.rsplit('/').next() {
            Some(slug) => slug,
            None => "",
        }
    }

    /// Resolves a wiki title to its full url
    ///
    /// # Arguments
    ///
    /// * `title` - the title of the wiki page
    ///
    /// # Returns
    ///
    /// - Ok(String) - owned wiki url
    /// - Err(WikiError::NetworkingError) - some network error
    pub fn resolve_wiki_url(title: &str) -> Result<String, WikiError> {
        let client: Arc<Client> = get_client();

        let base_url = "https://en.wikipedia.org/wiki/";
        let slug = title.replace(" ", "_");
        let url = base_url.to_owned() + &slug;

        match client.get(&url).send() {
            Ok(response) => {
                if response.status().is_success() {
                    Ok(url)
                } else {
                    Err(WikiError::NetworkingError(
                        format!("URL returned status: {}", response.status()).into(),
                    ))
                }
            }
            Err(e) => Err(WikiError::NetworkingError(
                "Failed to send request: {}".to_owned(),
            )),
        }
    }
}

fn handle_response(response: reqwest::blocking::Response) -> Result<String, WikiError> {
    debug!("Handling response...");
    if response.status().is_success() {
        debug!("Response successful. Extracting text...");

        Ok(response.text().map_err(|err| {
            WikiError::NetworkingError("Failed to get text from response".to_owned())
        })?)
    } else {
        debug!("Response failed with status: {}", response.status());
        Err(WikiError::NetworkingError(format!(
            "Failed to fetch page: HTTP {}",
            response.status()
        )))
    }
}

/// A struct representing an entire wiki page.
/// From an IR standpoint, this represents a graph node of a semantic network
/// It's outlinks are the `links` field. This does not contain backlinks, as this
/// library is built for dynamic traversal
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Page {
    pub title: String,
    pub links: Vec<HyperLink>,
    pub content: String,
}

/// A struct representing a hyperlink out of a wiki page, to another.
/// From an IR standpoint, this represents a graph edge of a semantic network
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct HyperLink {
    pub title: String,
    pub outlink: String,
}

fn process_content_recursive(
    element: scraper::ElementRef,
    raw_content: &mut String,
    links: &mut Vec<HyperLink>,
) {
    for node in element.children() {
        if let Some(text) = node.value().as_text() {
            raw_content.push_str(text);
        } else if let Some(elem) = scraper::ElementRef::wrap(node) {
            if elem.value().name() == "a" {
                if let Some(href) = elem.value().attr("href") {
                    let cur_outline = href.to_string();

                    if !cur_outline.starts_with("/wiki/") {
                        continue;
                    }

                    let link = HyperLink {
                        title: elem.text().collect::<String>(),
                        outlink: "https://en.wikipedia.org".to_string() + &cur_outline,
                    };
                    links.push(link);
                }
            } else {
                // process children elements
                process_content_recursive(elem, raw_content, links);
            }
        }
    }
}

///Processes a raw wikipedia fetch into a Page
///
/// # Arguments
///
/// * `element` - root element of wikipedia DOM
/// * 'page_title` - title of wikipedia page being processed
///
/// # Returns
///
/// Page struct representing the given wiki page
pub fn process_content(element: scraper::ElementRef, page_title: &str) -> Page {
    debug!("Processing content element...");
    let mut raw_content = String::new();
    let mut links = Vec::new();

    process_content_recursive(element, &mut raw_content, &mut links);

    // clean meta content is actually not a cheap function,
    // only wanna call it once here vs inside the recursive one
    let cleaned_content = clean_meta_content(&raw_content);

    Page {
        title: page_title.trim().to_owned(),
        content: cleaned_content,
        links: links,
    }
}

use regex::Regex;

/// Cleans the wikipedia meta content from a string
///
/// # Arguments
///
/// * `input` - Input content to clean
///
/// # Returns
///
/// String cleaned of wikipedia meta content
pub fn clean_meta_content(input: &str) -> String {
    debug!("Cleaning meta content...");
    let re_whitespace = Regex::new(r"\s+").unwrap();
    let cleaned_text = re_whitespace.replace_all(input, " ").to_string();

    let re_css = Regex::new(r"\.mw-.*?\{.*?\}").unwrap();
    let cleaned_text_no_css = re_css.replace_all(&cleaned_text, "").to_string();

    let clean_text_no_symbols = cleaned_text_no_css.replace("()", "").replace("[]", "");

    let re_trim = Regex::new(r"^\s+|\s+$").unwrap();
    let final_text = re_trim.replace_all(&clean_text_no_symbols, "").to_string();

    debug!("Meta content cleaned.");
    final_text
}

/// Removes non-semantic indicators from document
///
/// # Arguments
///
/// * `page` - page to clean
///
/// # Returns
///
/// A new, owned clean page with no non-semantic indicators
pub fn clean_document(page: &Page) -> Page {
    let stop_words: Vec<String> = STOP_WORDS.to_vec();

    debug!("Cleaning document...");
    let mut results: String = String::new();

    page.content
        .split_whitespace()
        .map(|word| word.trim())
        .filter(|word| word.is_ascii())
        .filter(|word| word.chars().all(|c| c.is_alphabetic()))
        .map(|word| word.to_ascii_lowercase())
        .inspect(|word| debug!("current word: {:?}", word))
        .filter(|word| !stop_words.contains(&word.to_string()))
        .map(|word| word.to_ascii_lowercase())
        .for_each(|word| {
            results.push_str(&word);
            results.push_str(" ");
        });

    debug!("Document cleaned.");

    Page {
        title: page.title.clone(),
        links: page.links.clone(),
        content: results,
    }
}

/// Convert a Page into its vector representation in a word embeddding vector space
///
/// # Arguments
///
/// * `page` - The page to convert
/// * `vocab` - shared vocabulary that you want to use
///
/// # Returns
///
/// An owned vector of floats containing ONLY the term-frequencies values
/// This notably does not contain the IDF information
/// 
pub fn page_to_vec(page: &Page, vocab: &HashMap<String, usize>) -> Vec<f64> {
    let content = clean_document(page).content;
    let words: Vec<&str> = content.split_whitespace().collect();

    let mut word_count = HashMap::new();
    for &word in &words {
        *word_count.entry(word.to_string()).or_insert(0) += 1;
    }

    let total_words = words.len() as f64;
    let mut vector = vec![0.0; vocab.len()];

    for (word, &count) in &word_count {
        if let Some(&index) = vocab.get(word) {
            vector[index] = count as f64 / total_words;
        }
    }

    vector
}
use rayon::prelude::*;
use stop_words::STOP_WORDS;

/// The cosine similarity between two vectors
///
/// # Arguments
///
/// * `vec1` - The first vector
/// * `vec2` - The second vector
///
/// # Returns
///
/// the cosine of the angle between the vectors -> [0-1)
pub fn cosine_sim(vec1: &Vec<f64>, vec2: &Vec<f64>) -> f64 {
    let dot_product: f64 = vec1
        .par_iter()
        .zip(vec2.par_iter())
        .map(|(a, b)| a * b)
        .sum();

    //par iter brrrrrrrrrrrrrrrr
    let magnitude1: f64 = vec1.par_iter().map(|x| x.powi(2)).sum::<f64>().sqrt();
    let magnitude2: f64 = vec2.par_iter().map(|x| x.powi(2)).sum::<f64>().sqrt();
    return dot_product / (magnitude1 * magnitude2);
}

/// Get the similarity of two pages
///
/// # Arguments
///
/// * `page1` - The first page to check
/// * `page2` - The second page to check
///
/// # Returns
///
/// The document similarity [0-1)
pub fn get_page_similarity(page1: &Page, page2: &Page) -> f64 {
    let mut vocab = HashMap::new();

    // need shared vocab now
    let mut vocab_len = 0;
    for page in &[page1, page2] {
        let content = clean_document(page).content;

        for word in content.split_whitespace() {
            vocab_len = vocab.len();

            vocab.entry(word.to_string()).or_insert(vocab_len);
        }
    }

    let vec1 = page_to_vec(page1, &vocab);
    let vec2 = page_to_vec(page2, &vocab);

    cosine_sim(&vec1, &vec2)
}

/// Get the most similar page from a set of pages
///
/// # Arguments
///
/// * `primary_page` - The page to check for similarity to
/// * `pages` - The set of pages to check against
///
/// # Returns
///
/// The ARGMAX of the most similar page
pub fn get_most_similar_page(primary_page: &Page, pages: &Vec<Page>) -> usize {
    let mut vocab = HashMap::new();


    let mut vocab_len = 0;
    // Build shared vocabulary from primary_page and all comparison pages
    for page in std::iter::once(primary_page).chain(pages.iter()) {
        let content = clean_document(page).content;

        for word in content.split_whitespace() {
            vocab_len = vocab.len();
            vocab.entry(word.to_string()).or_insert(vocab_len);
        }
    }

    let primary_vec = page_to_vec(primary_page, &vocab);

    let mut most_similar_index: usize = 0;
    let mut best_similarity: f64 = -1.0; // start at most dissimilar

    for (page_index, page) in pages.iter().enumerate() {
        let cur_vec = page_to_vec(page, &vocab);
        let cur_sim = cosine_sim(&primary_vec, &cur_vec);

        if cur_sim > best_similarity {
            best_similarity = cur_sim;
            most_similar_index = page_index;
        }
    }

    most_similar_index
}