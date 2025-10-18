use reqwest;
use scraper::{Html, Selector};
use serde_json::json;
use std::error::Error;

mod models;
mod scraper_logic;

use models::StockData;

/// Receives a stock code and returns a URL for Yahoo Finance.
fn build_url_from_code(code: &str) -> String {
    if code == "%5EDJI" {
        "https://finance.yahoo.co.jp/quote/%5EDJI".to_string()
    } else if code.ends_with(".O") { // For indices like Nikkei
        format!("https://finance.yahoo.co.jp/quote/{}", code)
    } else { // For standard TSE stocks
        format!("https://finance.yahoo.co.jp/quote/{}.T", code)
    }
}

/// Dynamically finds the name and its selector from the page.
async fn find_name_dynamically(document: &Html) -> Result<(Option<String>, String), Box<dyn Error>> {
    let mut found_name_selector: Option<String> = None;
    let mut found_name_text = String::new();

    let h2_selector = Selector::parse("h2").map_err(|e| models::ScraperError(format!("{:?}", e)))?;
    let mut best_candidate_selector = None;
    let mut fallback_candidate_selector = None;
    let mut best_candidate_text = None;
    let mut fallback_candidate_text = None;

    for element in document.select(&h2_selector) {
        let text = element.text().collect::<String>().trim().to_string();
        if !text.is_empty() && !text.chars().all(char::is_numeric) {
            if text.contains("(株)") || text == "NYダウ" || text == "日経平均株価" {
                best_candidate_selector = Some(scraper_logic::build_selector(&element));
                best_candidate_text = Some(text);
                break;
            }
            if fallback_candidate_selector.is_none() {
                fallback_candidate_selector = Some(scraper_logic::build_selector(&element));
                fallback_candidate_text = Some(text);
            }
        }
    }

    if best_candidate_selector.is_some() {
        found_name_selector = best_candidate_selector;
        found_name_text = best_candidate_text.unwrap_or_default();
    } else if fallback_candidate_selector.is_some() {
        found_name_selector = fallback_candidate_selector;
        found_name_text = fallback_candidate_text.unwrap_or_default();
    }

    Ok((found_name_selector, found_name_text))
}

/// Helper function to scrape a single field using a selector.
fn scrape_field(document: &Html, selector_opt: &Option<String>, field_name: &str) -> String {
    if let Some(selector_str) = selector_opt {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(element) = document.select(&selector).next() {
                let value = element.text().collect::<String>().trim().to_string();
                // println!("    - [Debug] Scraping '{}' with selector '{}' -> Found: '{}'", field_name, selector_str, value);
                return value;
            }
        }
    }
    // println!("    - [Debug] Scraping '{}': Selector not found or invalid.", field_name);
    String::new()
}


/// Scrapes a single stock page dynamically without any prior knowledge of the stock's name.
async fn scrape_dynamically(code: &str) -> Result<StockData, Box<dyn Error>> {
    let url = build_url_from_code(code);

    let response = reqwest::get(&url).await?;
    let body = response.text().await?;
    let document = Html::parse_document(&body);

    // 1. Find the name and its selector first.
    let (_name_selector_opt, name_text) = find_name_dynamically(&document).await?;

    if name_text.is_empty() {
        return Err(Box::new(models::ScraperError(
            "Could not dynamically find a valid name.".to_string(),
        )));
    }

    // 2. Use the found name as an anchor to find everything else.
    let anchor_name = &name_text;
    let zenjitsuhi_anchor = "前日比";

    // println!("  -> Found name '{}'. Finding other selectors...", anchor_name);
    let code_selector_opt = scraper_logic::find_text_pattern_selector_near_anchor(&document, anchor_name, "code").await?;
    let price_selector_opt = scraper_logic::find_price_selector_near_anchor(&document, anchor_name).await?;
    let ratio_selector_opt = scraper_logic::find_ratio_selector_near_zenjitsuhi(&document, zenjitsuhi_anchor).await?;
    let percent_selector_opt = scraper_logic::find_percent_selector_near_zenjitsuhi(&document, zenjitsuhi_anchor).await?;

    // 3. Scrape data using the found selectors.
    // println!("  -> Scraping final data from page...");
    let mut scraped_data = StockData::default();
    scraped_data.name = name_text;
    scraped_data.code = scrape_field(&document, &code_selector_opt, "code");
    scraped_data.price = scrape_field(&document, &price_selector_opt, "price");
    scraped_data.ratio = scrape_field(&document, &ratio_selector_opt, "ratio");
    scraped_data.percent = scrape_field(&document, &percent_selector_opt, "percent");

    // 4. Fill in missing data
    if scraped_data.code.is_empty() {
        scraped_data.code = code.to_string();
    }

    Ok(scraped_data)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let stock_codes = vec!["6758", "7203", "%5EDJI", "998407.O", "8279"];
    let mut all_stock_data: Vec<StockData> = Vec::new();

    println!("--- Running Dynamic Scraper ---");
    for code in stock_codes {
        println!("Scraping code: {}", code);
        match scrape_dynamically(code).await {
            Ok(data) => all_stock_data.push(data),
            Err(e) => eprintln!("  -> Error: {}", e),
        }
    }

    println!("\n--- Scraped Data ---");
    let scraped_data_json = json!(all_stock_data);
    println!("{}", serde_json::to_string_pretty(&scraped_data_json)?);

    Ok(())
}
