use scraper::{Html, Selector};
use reqwest::blocking::get;

// --- データ構造 ---
#[derive(Debug)]
struct StockData {
    name: String,
    code: String,
    price: String,
    change_abs: String, // 前日比（金額）
    change_pct: String, // 前日比（パーセント）
    update_time: String,
}

// --- ヘルパー関数：前日比の文字列を金額とパーセントに分割 ---
fn parse_change_string(combined: &str) -> (String, String) {
    if let Some(paren_index) = combined.find('(') {
        let abs = combined[..paren_index].trim().to_string();
        let pct_part = &combined[paren_index + 1..];
        let pct = if let Some(end_paren_index) = pct_part.find(')') {
            pct_part[..end_paren_index].trim().to_string()
        } else {
            "".to_string()
        };
        (abs, pct)
    } else {
        (combined.trim().to_string(), "".to_string())
    }
}

// --- 個別株価ページのスクレイピング関数 ---
fn scrape_stock_page_data(document: &Html) -> Result<StockData, Box<dyn std::error::Error>> {
    let container_sel = Selector::parse("div[class*='PriceBoard__main']").unwrap();
    let container = document.select(&container_sel).next().ok_or("Main container not found")?;

    let name_sel = Selector::parse("header h2").unwrap();
    let name = container
        .select(&name_sel)
        .next()
        .map(|e| e.text().collect::<String>().trim().to_string())
        .unwrap_or_default();

    let code_sel = Selector::parse("span[class*='PriceBoard__code']").unwrap();
    let code = container
        .select(&code_sel)
        .next()
        .map(|e| e.text().collect::<String>().trim().to_string())
        .unwrap_or_default();

    let price_sel =
        Selector::parse("span[class*='PriceBoard__price'] span[class*='StyledNumber__value']")
            .unwrap();
    let price = container
        .select(&price_sel)
        .next()
        .map(|e| e.text().collect::<String>().trim().to_string())
        .unwrap_or_default();

    let change_sel = Selector::parse("div[class*='PriceChangeLabel']").unwrap();
    let combined_change = container
        .select(&change_sel)
        .next()
        .map(|e| {
            e.text()
                .collect::<String>()
                .replace("前日比", "")
                .replace('\n', " ")
                .trim()
                .to_string()
        })
        .unwrap_or_default();
    let (change_abs, change_pct) = parse_change_string(&combined_change);

    let time_sel = Selector::parse("ul[class*='PriceBoard__times'] time").unwrap();
    let update_time = container
        .select(&time_sel)
        .next()
        .map(|e| e.text().collect::<String>().trim().to_string())
        .unwrap_or_default();

    Ok(StockData {
        name,
        code,
        price,
        change_abs,
        change_pct,
        update_time,
    })
}

// --- 指数ページ（^DJIなど）のスクレイピング関数 ---
fn scrape_index_data(document: &Html, code: &str) -> Result<StockData, Box<dyn std::error::Error>> {
    let name_sel = Selector::parse("h1").unwrap();
    let raw_name = document
        .select(&name_sel)
        .next()
        .map(|e| e.text().collect::<String>().trim().to_string())
        .unwrap_or_default();
    let name = raw_name.replace("の指数情報・推移", "").trim().to_string();

    let container_sel = Selector::parse("div[class*='_BasePriceBoard__main']").unwrap();
    let container = match document.select(&container_sel).next() {
        Some(c) => c,
        None => return Err(format!("Index container not found for {}.", code).into())
    };

    let price_block_sel = Selector::parse("div[class*='_BasePriceBoard__price']").unwrap();
    let price_block_text = container
        .select(&price_block_sel)
        .next()
        .map(|e| e.text().collect::<String>())
        .unwrap_or_default();

    let (price, combined_change) = {
        let change_label = "前日比";
        let time_label = "リアルタイム";

        if let Some(change_start_index) = price_block_text.find(change_label) {
            let price_str = price_block_text[..change_start_index].trim().to_string();
            let rest_of_string = &price_block_text[change_start_index + change_label.len()..];

            let change_str = if let Some(time_start_index) = rest_of_string.find(time_label) {
                rest_of_string[..time_start_index].trim().to_string()
            } else {
                rest_of_string.trim().to_string()
            };
            (price_str, change_str)
        } else {
            (price_block_text.trim().to_string(), "".to_string())
        }
    };
    let (change_abs, change_pct) = parse_change_string(&combined_change);

    let mut update_time = "".to_string();
    let list_items_sel = Selector::parse("ul li").unwrap();
    let mut found_realtime = false;
    for li in document.select(&list_items_sel) {
        let text = li.text().collect::<String>();
        if found_realtime {
            update_time = text.trim().to_string();
            break;
        }
        if text.contains("リアルタイム") {
            found_realtime = true;
        }
    }

    Ok(StockData {
        name,
        code: code.to_string(),
        price,
        change_abs,
        change_pct,
        update_time,
    })
}

// --- PriceBoard系ページ（日経平均, FXなど）のスクレイピング関数 ---
fn scrape_priceboard_data(document: &Html, code: &str) -> Result<StockData, Box<dyn std::error::Error>> {
    let container_sel = Selector::parse("div[class*='PriceBoard__main']").unwrap();
    let container = match document.select(&container_sel).next() {
        Some(c) => c,
        None => return Err(format!("PriceBoard container not found for {}.", code).into())
    };

    let name_sel = Selector::parse("header h2").unwrap();
    let name = container
        .select(&name_sel)
        .next()
        .map(|e| e.text().collect::<String>().trim().to_string())
        .unwrap_or_default();

    let price_sel =
        Selector::parse("span[class*='PriceBoard__price'] span[class*='StyledNumber__value']")
            .unwrap();
    let price = container
        .select(&price_sel)
        .next()
        .map(|e| e.text().collect::<String>().trim().to_string())
        .unwrap_or_default();

    let change_sel = Selector::parse("div[class*='PriceChangeLabel']").unwrap();
    let combined_change = container
        .select(&change_sel)
        .next()
        .map(|e| e.text().collect::<String>().replace("前日比", "").trim().to_string())
        .unwrap_or_default();
    let (change_abs, change_pct) = parse_change_string(&combined_change);

    let time_sel = Selector::parse("ul[class*='PriceBoard__times'] time").unwrap();
    let update_time = container
        .select(&time_sel)
        .next()
        .map(|e| e.text().collect::<String>().trim().to_string())
        .unwrap_or_default();

    Ok(StockData {
        name,
        code: code.to_string(),
        price,
        change_abs,
        change_pct,
        update_time,
    })
}

// --- 処理の振り分け関数 ---
fn scrape_data(code: &str) -> Result<StockData, Box<dyn std::error::Error>> {
    let url = format!("https://finance.yahoo.co.jp/quote/{}", code);
    let html = get(&url)?.text()?;
    let document = Html::parse_document(&html);

    if code.starts_with('^') {
        scrape_index_data(&document, code)
    } else if code.ends_with(".O") || code.ends_with("=X") {
        scrape_priceboard_data(&document, code)
    } else {
        scrape_stock_page_data(&document)
    }
}

// --- メイン処理 ---
fn main() {
    let stock_codes = vec!["^DJI", "998407.O", "USDJPY=X", "6758.T", "8729.T", "5016.T", "4755.T"];

    println!("--- 複数銘柄の株価情報取得を開始 ---");
    println!();

    for code in stock_codes {
        match scrape_data(code) {
            Ok(data) => {
                println!("--- {} ---", data.name);
                println!("  🏷️ 銘柄コード : {}", data.code);
                println!("  💰 株価     : {}", data.price);
                println!("  📉 前日比(金額) : {}", data.change_abs);
                println!("  📉 前日比(%)   : {}", data.change_pct);
                println!("  🕔 更新時間 : {}", data.update_time);
                println!();
            }
            Err(e) => {
                eprintln!("銘柄 {} の取得に失敗しました: {}", code, e);
                eprintln!();
            }
        }
    }

    println!("--- 全ての処理が完了しました ---");
}