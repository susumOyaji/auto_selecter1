use reqwest;
use scraper::{Html};
use serde_json::json;
use std::error::Error;
use std::env;

mod models;
mod scraper_logic;

use models::StockData;

/// Receives a stock code and returns a URL for Yahoo Finance.
fn build_url_from_code(code: &str) -> String {
    if code == "%5EDJI" || code == "^DJI" || code == "DJI" { // Added || code == "DJI"
        "https://finance.yahoo.co.jp/quote/%5EDJI".to_string()
    } else if code.ends_with(".O") || code.ends_with("=FX") {
        format!("https://finance.yahoo.co.jp/quote/{}", code)
    } else { // For standard TSE stocks
        format!("https://finance.yahoo.co.jp/quote/{}.T", code)
    }
}

/// Scrapes a single stock page dynamically without any prior knowledge of the stock's name.
async fn scrape_dynamically(code: &str) -> Result<StockData, Box<dyn Error>> {
    let url = build_url_from_code(code);

    let response = reqwest::get(&url).await?;
    let body = response.text().await?;
    let document = Html::parse_document(&body);

    // 1. Find the name and its selector first.
    let (_name_selector_opt, name_text) = scraper_logic::find_name_dynamically(&document).await?;

    if name_text.is_empty() {
        return Err(Box::new(models::ScraperError(
            "Could not dynamically find a valid name.".to_string(),
        )));
    }

    // 2. Use the found name as an anchor to find everything else.
    let anchor_name = &name_text;

    let code_selector_opt = scraper_logic::find_text_pattern_selector_near_anchor(&document, anchor_name, "code").await?;
    
    let price_selector_opt;
    let change_selector_opt;
    let change_percent_selector_opt;
    let update_time_selector_opt;

    if code.ends_with("=FX") {
        // FX-specific logic
        price_selector_opt = scraper_logic::find_fx_price_selector(&document).await?;
        change_selector_opt = scraper_logic::find_fx_change_selector(&document).await?;
        change_percent_selector_opt = None; // User requested to not scrape change_percent for FX
        update_time_selector_opt = scraper_logic::find_fx_update_time_selector(&document).await?;
    } else {
        // Stock-specific logic
        let zenjitsuhi_anchor = "前日比";
        price_selector_opt = scraper_logic::find_stock_price_selector(&document, anchor_name).await?;
        change_selector_opt = scraper_logic::find_stock_change_selector(&document, zenjitsuhi_anchor).await?;
        change_percent_selector_opt = scraper_logic::find_stock_change_percent_selector(&document, zenjitsuhi_anchor).await?;
        update_time_selector_opt = scraper_logic::find_stock_update_time_selector(&document).await?;
    }

    // 3. Scrape data using the found selectors.
    let mut scraped_data = StockData::default();
    scraped_data.name = name_text;
    scraped_data.code = scraper_logic::scrape_field(&document, &code_selector_opt, "code");
    scraped_data.price = scraper_logic::scrape_field(&document, &price_selector_opt, "price");
    scraped_data.change = scraper_logic::scrape_field(&document, &change_selector_opt, "change");
    scraped_data.change_percent = scraper_logic::scrape_field(&document, &change_percent_selector_opt, "change_percent");
    scraped_data.update_time = scraper_logic::scrape_field(&document, &update_time_selector_opt, "update_time");

    // 4. Fill in missing data
    if scraped_data.code.is_empty() {
        scraped_data.code = code.to_string();
    }

    Ok(scraped_data)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let stock_codes: Vec<String> = env::args().skip(1)
        .flat_map(|arg| {
            // split が借用イテレータを返すので、ここで所有権を持つ Vec<String> に集める
            arg.split(',').map(str::to_string).collect::<Vec<String>>()
        }) // カンマで分割し、すべてを収集
        .collect();

    if stock_codes.is_empty() {
        eprintln!("Usage: auto_selecter1 <stock_code_1> <stock_code_2> ...");
        eprintln!("Example: auto_selecter1 6758 7203 USDJPY=FX");
        return Ok(())
    }

    let mut all_stock_data: Vec<StockData> = Vec::new();

    println!("--- Running Dynamic Scraper ---");
    for code in &stock_codes {
        println!("Scraping code: {}", code);
        match scrape_dynamically(code).await {
            Ok(data) => all_stock_data.push(data),
            Err(e) => eprintln!("  -> Error scraping {}: {}", code, e),
        }
    }

    println!("\n--- Scraped Data ---");
    let scraped_data_json = json!(all_stock_data);
    println!("{}", serde_json::to_string_pretty(&scraped_data_json)?);

    Ok(())
}