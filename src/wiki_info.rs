use std::collections::HashMap;

use scraper::{Html, Selector};

mod stop_words;

// singleton module wrapper
mod client {
    use lazy_static::lazy_static;
    use reqwest::blocking::Client;
    use std::sync::{Arc, Mutex};

    struct ClientSingleton {
        blocking_client: Arc<reqwest::blocking::Client>,
    }

    impl ClientSingleton {
        fn new() -> Self {
            println!("Initializing ClientSingleton...");
            ClientSingleton {
                blocking_client: Arc::new(Client::new()),
            }
        }

        // private version of this fn
        fn get_client(&self) -> Arc<Client> {
            println!("Retrieving client from ClientSingleton...");
            Arc::clone(&self.blocking_client)
        }
    }

    // lazy static init of singleton
    lazy_static! {
        static ref CLIENT_INSTANCE: Mutex<ClientSingleton> = {
            println!("Creating CLIENT_INSTANCE...");
            std::sync::Mutex::new(ClientSingleton::new())
        };
    }

    pub fn get_client() -> Arc<Client> {
        println!("Acquiring lock on CLIENT_INSTANCE...");
        CLIENT_INSTANCE
            .lock()
            .expect("Failed to acquire lock on ClientSingleton") // ehhhh might wanna fix this
            .get_client()
    }
}

// need to figure out str vs string for public apis
pub fn page_from_title(title: &str) -> Result<Page, Box<dyn std::error::Error>> {
    println!("parse_parse_from_title called...");

    let url = url_utils::resolve_wiki_url(title)?;

    return page_from_url(&url);
}

pub fn page_from_url(url: &str) -> Result<Page, Box<dyn std::error::Error>> {
    println!("parse_page_from_url called with url: {}", url);
    let client = client::get_client();

    println!("Sending request to URL: {}", url);
    let response = client
        .get(url)
        // lie about user agents lol
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36",
        )
        .send()?;

    println!("Response received from URL: {}", url);
    let html_content = handle_response(response)?;

    println!("Parsing HTML content...");
    let document = Html::parse_document(&html_content);

    // this wierd selector is what gets the actual body from a page
    let selector = Selector::parse("div.mw-content-container main#content").unwrap();

    match document.select(&selector).next() {
        Some(content) => {
            println!("Content successfully selected. Processing content...");

            let title = url_utils::extract_slug(url)
                .split("_")
                .fold(String::new(), |a, b| a + b + " ");

            Ok(process_content(content, &title))
        }
        None => {
            println!("Failed to select content from document.");
            Err("Could not process content".to_string().into())
        }
    }
}

pub mod url_utils {
    use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
    use reqwest::{blocking::Client, header::LOCATION};
    use std::sync::Arc;

    use super::client::get_client;

    // util for title extraction
    pub fn extract_slug(url: &str) -> &str {
        // last elem
        match url.rsplit('/').next() {
            Some(slug) => slug,
            None => "",
        }
    }

    pub fn resolve_wiki_url(title: &str) -> Result<String, Box<dyn std::error::Error>> {
        let client: Arc<Client> = get_client();

        // wiki links have dumb special character to handle 
        let encoded_title = utf8_percent_encode(title, NON_ALPHANUMERIC).to_string();
        let url = format!("https://en.wikipedia.org/wiki/{}", encoded_title);

        match client.get(&url).send() {
            Ok(res) if res.status().is_redirection() => {
                res.headers()
                    .get(LOCATION)
                    .and_then(|redirect_url| redirect_url.to_str().ok())
                    .map(|redirect_str| redirect_str.to_owned())
                    .ok_or_else(|| "Could not resolve url".into())
                    // this is just like s js promise chain
            }
            Ok(res) => Ok(res.url().as_str().to_owned()), 
            Err(_) => Err("Could not process content".into()), 
        }
    }
}

fn handle_response(
    response: reqwest::blocking::Response,
) -> Result<String, Box<dyn std::error::Error>> {
    println!("Handling response...");
    if response.status().is_success() {
        println!("Response successful. Extracting text...");
        Ok(response.text()?)
    } else {
        println!("Response failed with status: {}", response.status());
        Err(format!("Failed to fetch page: HTTP {}", response.status()).into())
    }
}

#[derive(Debug)]
pub struct Page {
    pub title: String,
    pub links: Vec<HyperLink>,
    pub content: String, // own your strings cuz I hate lifetimes
}

#[derive(Debug)]
pub struct HyperLink {
    pub title: String,
    pub outlink: String,
}
pub fn process_content_recursive(
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

pub fn process_content(element: scraper::ElementRef, page_title: &str) -> Page {
    println!("Processing content element...");
    let mut raw_content = String::new();
    let mut links = Vec::new();

    process_content_recursive(element, &mut raw_content, &mut links);

    // clean meta content is actually not a cheap function, only wanna call it once here vs inside the recursive one
    let cleaned_content = clean_meta_content(&raw_content);

    Page {
        title: page_title.trim().to_owned(),
        content: cleaned_content,
        links: links,
    }
}

use regex::Regex;

fn clean_meta_content(input: &str) -> String {
    println!("Cleaning meta content...");
    let re_whitespace = Regex::new(r"\s+").unwrap();
    let cleaned_text = re_whitespace.replace_all(input, " ").to_string();

    let re_css = Regex::new(r"\.mw-.*?\{.*?\}").unwrap();
    let cleaned_text_no_css = re_css.replace_all(&cleaned_text, "").to_string();

    let clean_text_no_symbols = cleaned_text_no_css.replace("()", "").replace("[]", "");

    let re_trim = Regex::new(r"^\s+|\s+$").unwrap();
    let final_text = re_trim.replace_all(&clean_text_no_symbols, "").to_string();

    println!("Meta content cleaned.");
    final_text
}

// removes non-semantic indicators from document
pub fn clean_document(page: &Page) -> String {


    let stop_words: Vec<String> = STOP_WORDS.to_vec();

    println!("Cleaning document...");
    let mut results: String = String::new();

    page.content
        .split_whitespace()
        .map(|word| word.trim())
        .filter(|word| word.is_ascii())
        .filter(|word| word.chars().all(|c| c.is_alphabetic()))
        .map(|word| word.to_ascii_lowercase())
        // .inspect(|word| println!("current word: {:?}", word))
        .filter(|word| !stop_words.contains(&word.to_string()))
        .map(|word| word.to_ascii_lowercase())
        .for_each(|word| {
            results.push_str(&word);
            results.push_str(" ");
        });

    println!("Document cleaned.");
    results
}

pub fn page_to_vec(page: &Page) -> Vec<f64> {


    println!("Converting page to vector...");
    let content = clean_document(page);

    let content_vec: Vec<&str> = content.split_whitespace().collect();

    let mut word_count = HashMap::new();
    for &word in &content_vec {
        *word_count.entry(word).or_insert(0) += 1;
    }

    let total_words = content_vec.len() as f64;
    let mut term_frequencies = Vec::new();
    for &word in &content_vec {
        if let Some(&count) = word_count.get(word) {
            term_frequencies.push(count as f64 / total_words);
        }
    }

    println!("Page converted to vector.");
    term_frequencies
}

use rayon::prelude::*;
use stop_words::STOP_WORDS;

fn cosine_sim(vec1: &Vec<f64>, vec2: &Vec<f64>) -> f64 {
    let dot_product: f64 = vec1
        .par_iter()
        .zip(vec2.par_iter())
        .map(|(a, b)| a * b)
        .sum();

    //brrrrrrrrrrrrrrrr
    let magnitude1: f64 = vec1.par_iter().map(|x| x.powi(2)).sum::<f64>().sqrt();
    let magnitude2: f64 = vec2.par_iter().map(|x| x.powi(2)).sum::<f64>().sqrt();
    return dot_product / (magnitude1 * magnitude2);
}

pub fn get_page_similarity(page1: &Page, page2: &Page) -> f64 {
    let vec1 = page_to_vec(page1);
    let vec2 = page_to_vec(page2);

    return cosine_sim(&vec1, &vec2);
}

// argmax = index dont forget that
// ARGMAX
pub fn get_most_similar_page(primary_page: &Page, pages: &Vec<Page>) -> usize {
    // i kinda see why sklearn has this take 2 vectors and return 1 similarity vector now lol

    let primary_vec = page_to_vec(&primary_page);

    let mut most_similar_index: usize = 0;
    let mut best_similarity: f64 = 0.0;

    for page_index in 0..pages.len() {
        let cur_sim = cosine_sim(&primary_vec, &page_to_vec(&pages[page_index]));

        if cur_sim > best_similarity {
            best_similarity = cur_sim;
            most_similar_index = page_index;
        }
    }

    return most_similar_index;
}
