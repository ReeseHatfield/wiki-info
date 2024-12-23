// I think we'll want this to be fully synchronous
// gonna use blocking apis for now, unless we need async somewhere

// goal should be no networking in the consumer api
mod wiki_info;

use wiki_info::{get_most_similar_page, page_from_title, Page};

fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    println!("Most Similar page to Paris is {:?}", pages_to_check[most_similar_page].title);

    Ok(())
}
