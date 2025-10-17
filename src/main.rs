use reqwest;
use scraper::{ElementRef, Html, Selector};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::error::Error;

mod static_scraper;
// ...existing code...
use crate::static_scraper::scrape_statically;

// ...existing code...
#[derive(Debug)]
pub struct ScraperError(pub String);

impl std::fmt::Display for ScraperError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for ScraperError {}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct StockData {
    pub code: String,
    pub name: String,
    pub price: String,
    pub ratio: String,
    pub percent: String,
    pub selector_type: String,
    // Add other fields you want to scrape
}

#[derive(Debug, Default)]
struct ScrapedSelectors {
    name_selector: Option<String>,
    code_selector: Option<String>,
    price_selector: Option<String>,
    ratio_selector: Option<String>,
    percent_selector: Option<String>,
}


struct AnchorTexts {
    // Existing
    dow_name: &'static str,
    dow_code: &'static str,
    zenjitsuhi: &'static str,
    nikkei_code: &'static str,

    // Error messages
    error_fetching_static: &'static str,
    error_fetching_dynamic: &'static str,
    err_dyn_code_sel_not_found_dow: &'static str,
    err_dyn_name_sel_not_found_dow: &'static str,
    err_dyn_price_sel_not_found_dow: &'static str,
    err_dyn_ratio_sel_not_found_dow: &'static str,
    err_dyn_percent_sel_not_found_dow: &'static str,
    err_invalid_code_sel_dow: &'static str,
    err_invalid_name_sel_dow: &'static str,
    err_invalid_price_sel_dow: &'static str,
    err_invalid_ratio_sel_dow: &'static str,
    err_invalid_percent_sel_dow: &'static str,
    err_dyn_code_sel_not_found_stock: &'static str,
    err_dyn_name_sel_not_found_stock: &'static str,
    err_dyn_price_sel_not_found_stock: &'static str,
    err_dyn_ratio_sel_not_found_stock: &'static str,
    err_dyn_percent_sel_not_found_stock: &'static str,
    err_invalid_code_sel_stock: &'static str,
    err_invalid_name_sel_stock: &'static str,
    err_invalid_price_sel_stock: &'static str,
    err_invalid_ratio_sel_stock: &'static str,
    err_invalid_percent_sel_stock: &'static str,
    err_dyn_name_sel_not_found_stock_fallback: &'static str,
    err_parse_selector: &'static str, // For general selector parsing errors

    // URLs and URL parts
    yahoo_finance_base_url: &'static str,
    stock_code_suffix: &'static str,
    dow_url_path: &'static str, // "%5EDJI"

    // Known stock names and codes
    sony_code: &'static str,
    sony_name: &'static str,
    toyota_code: &'static str,
    toyota_name: &'static str,
    nikkei_name: &'static str,

    // Selector types
    selector_type_dynamic: &'static str,
    selector_type_static: &'static str,

    // Selector patterns
    h2_price_board_name_selector: &'static str,
    h2_tag_selector: &'static str,
    pattern_type_code: &'static str,
    pattern_type_price: &'static str,

    // Main function output
    main_logic_start: &'static str,
    static_results_header: &'static str,
    dynamic_results_header: &'static str,
    output_code: &'static str,
    output_name: &'static str,
    output_price: &'static str,
    output_ratio: &'static str,
    output_percent: &'static str,
    output_selector_type: &'static str,
    output_separator: &'static str,
    error_occurred: &'static str,
    main_logic_end: &'static str,

    // Test related
    test_url_sony: &'static str,
    test_url_nikkei: &'static str,
}





impl AnchorTexts {
    const fn new() -> Self {
        AnchorTexts {
            dow_name: "NYダウ",
            dow_code: "^DJI",
            zenjitsuhi: "前日比",
            nikkei_code: "998407.O",

            // Error messages
            error_fetching_static: "Error fetching static data for: ",
            error_fetching_dynamic: "Error fetching dynamic data for: ",
            err_dyn_code_sel_not_found_dow: "Dynamic code selector not found for DOW",
            err_dyn_name_sel_not_found_dow: "Dynamic name selector not found for DOW",
            err_dyn_price_sel_not_found_dow: "Dynamic price selector not found for DOW",
            err_dyn_ratio_sel_not_found_dow: "Dynamic ratio selector not found for DOW",
            err_dyn_percent_sel_not_found_dow: "Dynamic percent selector not found for DOW",
            err_invalid_code_sel_dow: "Invalid code selector for DOW: ",
            err_invalid_name_sel_dow: "Invalid name selector for DOW: ",
            err_invalid_price_sel_dow: "Invalid price selector for DOW: ",
            err_invalid_ratio_sel_dow: "Invalid ratio selector for DOW: ",
            err_invalid_percent_sel_dow: "Invalid percent selector for DOW: ",
            err_dyn_code_sel_not_found_stock: "Dynamic code selector not found for stock",
            err_dyn_name_sel_not_found_stock: "Dynamic name selector not found for stock",
            err_dyn_price_sel_not_found_stock: "Dynamic price selector not found for stock",
            err_dyn_ratio_sel_not_found_stock: "Dynamic ratio selector not found for stock",
            err_dyn_percent_sel_not_found_stock: "Dynamic percent selector not found for stock",
            err_invalid_code_sel_stock: "Invalid code selector for stock: ",
            err_invalid_name_sel_stock: "Invalid name selector for stock: ",
            err_invalid_price_sel_stock: "Invalid price selector for stock: ",
            err_invalid_ratio_sel_stock: "Invalid ratio selector for stock: ",
            err_invalid_percent_sel_stock: "Invalid percent selector for stock: ",
            err_dyn_name_sel_not_found_stock_fallback: "Dynamic name selector not found for stock and no suitable h2 found.",
            err_parse_selector: "",

            // URLs and URL parts
            yahoo_finance_base_url: "https://finance.yahoo.co.jp/quote/",
            stock_code_suffix: ".T",
            dow_url_path: "%5EDJI",

            // Known stock names and codes
            sony_code: "6758",
            sony_name: "ソニーグループ(株)",
            toyota_code: "7203",
            toyota_name: "トヨタ自動車(株)",
            nikkei_name: "日経平均株価",

            // Selector types
            selector_type_dynamic: "dynamic",
            selector_type_static: "static",

            // Selector patterns
            h2_price_board_name_selector: "h2.PriceBoard__name__166W",
            h2_tag_selector: "h2",
            pattern_type_code: "code",
            pattern_type_price: "price",

            // Main function output
            main_logic_start: "--- Running Original Main Logic ---",
            static_results_header: "\n--- Static Selector Results ---",
            dynamic_results_header: "\n--- Dynamic Selector Results ---",
            output_code: "コード: ",
            output_name: "名前: ",
            output_price: "価格: ",
            output_ratio: "変化: ",
            output_percent: "変化率: ",
            output_selector_type: "セレクタータイプ: ",
            output_separator: "---",
            error_occurred: "エラーが発生しました: ",
            main_logic_end: "--- End of Original Main Logic ---\n",

            // Test related
            test_url_sony: "https://finance.yahoo.co.jp/quote/6758.T",
            test_url_nikkei: "https://finance.yahoo.co.jp/quote/998407.O",
        }
    }
}





#[derive(Deserialize)]


struct ScrapingRequest {


    static_codes: Vec<String>,


    dynamic_codes: Vec<String>,


}





pub async fn fetch_data_rust(codes_json: String) -> Result<String, Box<dyn Error>> {


    fetch_and_scrape_multiple(&codes_json).await


}





async fn fetch_and_scrape_multiple(codes_json: &str) -> Result<String, Box<dyn std::error::Error>> {


    let request: ScrapingRequest = serde_json::from_str(codes_json)?;


    let mut all_stock_data: Vec<StockData> = Vec::new();





    for code in request.static_codes {


        if let Ok(stock_info) = scrape_statically(&code).await {


            all_stock_data.push(stock_info);


        } else {


            eprintln!("{}{}", anchors.error_fetching_static, code);


        }


    }





    for code in request.dynamic_codes {


        if let Ok(stock_info) = scrape_dynamically(&code).await {


            all_stock_data.push(stock_info);


        } else {


            eprintln!("{}{}", anchors.error_fetching_dynamic, code);


        }


    }





    let scraped_data = json!(all_stock_data);


    Ok(scraped_data.to_string())


}





async fn scrape_dynamically(code: &str) -> Result<StockData, Box<dyn Error>> {
    let anchors = AnchorTexts::new();
    match code {
        anchors.dow_url_path => fetch_and_scrape_dow_dynamic().await,
        _ => {
            let url = if code == anchors.nikkei_code {
                format!("{}{}", anchors.yahoo_finance_base_url, code)
            } else {
                format!("{}{}{}", anchors.yahoo_finance_base_url, code, anchors.stock_code_suffix)
            };
            let known_name = match code {
                anchors.sony_code => anchors.sony_name,
                anchors.toyota_code => anchors.toyota_name,
                anchors.nikkei_code => anchors.nikkei_name,
                _ => code,
            };
            fetch_and_scrape_stock_dynamic(&url, known_name).await
        }
    }
}











async fn fetch_and_scrape_dow_dynamic() -> Result<StockData, Box<dyn Error>> {
    let anchors = AnchorTexts::new();
    let url = format!("{}{}", anchors.yahoo_finance_base_url, anchors.dow_url_path); // NYダウ平均のURL get_dow_dynamic_selectors

    let response = reqwest::get(url).await?;
    let body = response.text().await?;
    let document = Html::parse_document(&body);

    let selectors = get_dow_dynamic_selectors(&document).await;

    let code_selector_str = selectors.code_selector.ok_or_else(|| ScraperError(anchors.err_dyn_code_sel_not_found_dow.to_string()))?;
    let name_selector_str = selectors.name_selector.ok_or_else(|| ScraperError(anchors.err_dyn_name_sel_not_found_dow.to_string()))?;
    let price_selector_str = selectors.price_selector.ok_or_else(|| ScraperError(anchors.err_dyn_price_sel_not_found_dow.to_string()))?;
    let ratio_selector_str = selectors.ratio_selector.ok_or_else(|| ScraperError(anchors.err_dyn_ratio_sel_not_found_dow.to_string()))?;
    let percent_selector_str = selectors.percent_selector.ok_or_else(|| ScraperError(anchors.err_dyn_percent_sel_not_found_dow.to_string()))?;

    let code_selector = Selector::parse(&code_selector_str).map_err(|e| ScraperError(format!("{}{:?}", anchors.err_invalid_code_sel_dow, e)))?;
    let name_selector = Selector::parse(&name_selector_str).map_err(|e| ScraperError(format!("{}{:?}", anchors.err_invalid_name_sel_dow, e)))?;
    let price_selector = Selector::parse(&price_selector_str).map_err(|e| ScraperError(format!("{}{:?}", anchors.err_invalid_price_sel_dow, e)))?;
    let ratio_selector = Selector::parse(&ratio_selector_str).map_err(|e| ScraperError(format!("{}{:?}", anchors.err_invalid_ratio_sel_dow, e)))?;
    let percent_selector = Selector::parse(&percent_selector_str).map_err(|e| ScraperError(format!("{}{:?}", anchors.err_invalid_percent_sel_dow, e)))?;

    let code = document.select(&code_selector).next().map(|n| n.text().collect::<String>()).unwrap_or_default();
    let name = document.select(&name_selector).next().map(|n| n.text().collect::<String>()).unwrap_or_default();
    let price = document.select(&price_selector).next().map(|n| n.text().collect::<String>()).unwrap_or_default();
    let ratio = document.select(&ratio_selector).next().map(|n| n.text().collect::<String>()).unwrap_or_default();
    let percent = document.select(&percent_selector).next().map(|n| n.text().collect::<String>()).unwrap_or_default();

    Ok(StockData {
        code,
        name,
        price,
        ratio,
        percent,
        selector_type: anchors.selector_type_dynamic.to_string(),
    })
}





async fn fetch_and_scrape_stock_dynamic(url: &str, known_name: &str) -> Result<StockData, Box<dyn std::error::Error>> {


    let response = reqwest::get(url).await?;


    let body = response.text().await?;


    let document = Html::parse_document(&body);





    let selectors = get_stock_dynamic_selectors(&document, known_name).await?;





    let code_selector_str = selectors.code_selector.ok_or_else(|| ScraperError("Dynamic code selector not found for stock".to_string()))?;


    let name_selector_str = selectors.name_selector.ok_or_else(|| ScraperError("Dynamic name selector not found for stock".to_string()))?;


    let price_selector_str = selectors.price_selector.ok_or_else(|| ScraperError("Dynamic price selector not found for stock".to_string()))?;


    let ratio_selector_str = selectors.ratio_selector.ok_or_else(|| ScraperError("Dynamic ratio selector not found for stock".to_string()))?;


    let percent_selector_str = selectors.percent_selector.ok_or_else(|| ScraperError("Dynamic percent selector not found for stock".to_string()))?;





    let code_selector = Selector::parse(&code_selector_str).map_err(|e| ScraperError(format!("Invalid code selector for stock: {:?}", e)))?;


    let name_selector = Selector::parse(&name_selector_str).map_err(|e| ScraperError(format!("Invalid name selector for stock: {:?}", e)))?;


    let price_selector = Selector::parse(&price_selector_str).map_err(|e| ScraperError(format!("Invalid price selector for stock: {:?}", e)))?;


    let ratio_selector = Selector::parse(&ratio_selector_str).map_err(|e| ScraperError(format!("Invalid ratio selector for stock: {:?}", e)))?;


    let percent_selector = Selector::parse(&percent_selector_str).map_err(|e| ScraperError(format!("Invalid percent selector for stock: {:?}", e)))?;





    let code = document.select(&code_selector).next().map(|n| n.text().collect::<String>()).unwrap_or_default();


    let name = document.select(&name_selector).next().map(|n| n.text().collect::<String>()).unwrap_or_default();


    let price = document.select(&price_selector).next().map(|n| n.text().collect::<String>()).unwrap_or_default();


    let ratio = document.select(&ratio_selector).next().map(|n| n.text().collect::<String>()).unwrap_or_default();


    let percent = document.select(&percent_selector).next().map(|n| n.text().collect::<String>()).unwrap_or_default();





    Ok(StockData {


        code,


        name,


        price,


        ratio,


        percent,


        selector_type: "dynamic".to_string(),


    })


}





/// Finds a CSS selector dynamically for an element containing the given anchor text.


async fn find_dynamic_selector(


    document: &Html,


    anchor_text: &str,


) -> Result<Option<String>, Box<dyn std::error::Error>> {


    for node in document.root_element().descendants() {


        if let Some(text_node) = node.value().as_text() {


            if text_node.trim() == anchor_text {


                if let Some(parent) = node.parent() {


                    if let Some(element) = ElementRef::wrap(parent) {


                        return Ok(Some(build_selector(&element)));


                    }


                }


            }


        }


    }


    Ok(None)


}





/// Finds a CSS selector for a text node matching a pattern, near an anchor element.


async fn find_text_pattern_selector_near_anchor(


    document: &Html,


    anchor_text: &str,


    pattern_type: &str,


) -> Result<Option<String>, Box<dyn std::error::Error>> {





    const MAX_LEVELS: usize = 4;


    let mut search_area = None;


    // 1. Find anchor and search area


    for node in document.root_element().descendants() {


        if let Some(text_node) = node.value().as_text() {


            if text_node.trim() == anchor_text {


                let mut ancestor = None;


                let mut current = node.parent();


                for _ in 0..MAX_LEVELS {


                    if let Some(parent) = current {


                        if let Some(element) = ElementRef::wrap(parent) {


                            ancestor = Some(element);


                        }


                        current = parent.parent();


                    } else {


                        break;


                    }


                }


                search_area = ancestor;


                break;


            }


        }


    }





    // 2. Find pattern in text nodes


    if let Some(area) = search_area {


        for node in area.descendants() {


            if let Some(text_node) = node.value().as_text() {


                let trimmed_text = text_node.trim();


                let is_match = match pattern_type {


                    "code" => trimmed_text.len() == 4 && trimmed_text.chars().all(char::is_numeric),


                    "price" => trimmed_text.len() >= 4 && trimmed_text.chars().all(|c| c.is_numeric() || c == ','),


                    _ => false,


                };





                if is_match {


                    if let Some(parent) = node.parent().and_then(ElementRef::wrap) {


                        return Ok(Some(build_selector(&parent)));


                    }


                }


            }


        }


    }





    Ok(None)


}





/// "前日比"の近くにあるパーセント値要素のセレクターを抽出する


async fn find_percent_selector_near_zenjitsuhi(


    document: &Html,


    anchor_text: &str,


    max_levels: usize,


) -> Result<Option<String>, Box<dyn std::error::Error>> {





    // 1. "前日比"のノードを探し、探索範囲となる祖先要素を見つける


    let mut search_area = None;


    'outer: for node in document.root_element().descendants() {


        if let Some(text_node) = node.value().as_text() {


            if text_node.trim() == anchor_text {


                let mut current = node.parent();


                for _ in 0..max_levels {


                    if let Some(parent_node) = current {


                        if let Some(parent_element) = ElementRef::wrap(parent_node) {


                            search_area = Some(parent_element);


                            current = parent_node.parent();


                        }


                        else {


                            break;


                        }


                    }


                    else {


                        break;


                    }


                }


                break 'outer;


            }


        }


    }





    // 2. 探索範囲内でパーセント値っぽい要素を探す


    if let Some(area) = search_area {


        let selector = Selector::parse("*").map_err(|e| ScraperError(format!("{:?}", e)))?;


        let mut candidates = Vec::new();





        // ElementRef::select を使って子孫要素をイテレートし、候補をすべて集める


        for element in area.select(&selector) {


            let text = element.text().collect::<String>();


            let trimmed = text.trim();





            // パーセント値判定をより厳密にする


            if trimmed.contains('%') &&


               trimmed.contains('(') &&


               trimmed.contains(')') &&


               trimmed.chars().any(|c| c.is_numeric()) &&


               !trimmed.contains("前日比")


            {


                candidates.push(element);


            }


        }





        // 候補の中から最も深くネストされた（最後の）要素を選ぶ


        if let Some(best_candidate) = candidates.last() {


            let selector_str = build_selector(best_candidate);


            if selector_str.contains('.') {


                return Ok(Some(selector_str));


            }


        }


    }





    Ok(None)


}





/// "前日比"の近くにある変動幅（絶対値）の要素のセレクターを抽出する


async fn find_ratio_selector_near_zenjitsuhi(


    document: &Html,


    anchor_text: &str,


    max_levels: usize,


) -> Result<Option<String>, Box<dyn Error>> {





    // 1. "前日比"のノードを探し、探索範囲となる祖先要素を見つける


    let mut search_area = None;


    'outer: for node in document.root_element().descendants() {


        if let Some(text_node) = node.value().as_text() {


            if text_node.trim() == anchor_text {


                let mut current = node.parent();


                for _ in 0..max_levels {


                    if let Some(parent_node) = current {


                        if let Some(parent_element) = ElementRef::wrap(parent_node) {


                            search_area = Some(parent_element);


                            current = parent_node.parent();


                        }


                        else {


                            break;


                        }


                    }


                    else {


                        break;


                    }


                }


                break 'outer;


            }


        }


    }





    // 2. 探索範囲内で変動幅っぽい要素を探す


    if let Some(area) = search_area {


        let selector = Selector::parse("*").map_err(|e| ScraperError(format!("{:?}", e)))?;


        // ElementRef::select を使って子孫要素をイテレートする


        for element in area.select(&selector) {


            let text = element.text().collect::<String>();


            let trimmed = text.trim();





            // 変動幅の判定（"+" or "-"で始まり、数字が続き、"%"を含まない）


            if (trimmed.starts_with('+') || trimmed.starts_with('-'))


                && !trimmed.contains('%')


                && trimmed.len() > 1


            {


                let after_sign = &trimmed[1..].replace(",", "");


                if after_sign.parse::<f64>().is_ok() {


                    let selector_str = build_selector(&element);


                     // あまりに汎用的なセレクターは避ける (例: "span")


                    if selector_str.contains('.') {


                        return Ok(Some(selector_str));


                    }


                }


            }


        }


    }





    Ok(None)


}





/// Finds a CSS selector for the price, which is assumed to be a prominent numeric value


/// near the anchor text element.


async fn find_price_selector_near_anchor(


    document: &Html,


    anchor_text: &str,


    max_levels: usize,


) -> Result<Option<String>, Box<dyn std::error::Error>> {





    // 1. Find the anchor text node and the search area (ancestor element)


    let mut search_area = None;


    'outer: for node in document.root_element().descendants() {


        if let Some(text_node) = node.value().as_text() {


            if text_node.trim() == anchor_text {


                let mut current = node.parent();


                for _ in 0..max_levels {


                    if let Some(parent_node) = current {


                        if let Some(parent_element) = ElementRef::wrap(parent_node) {


                            search_area = Some(parent_element);


                            current = parent_node.parent();


                        }


                        else {


                            break;


                        }


                    }


                    else {


                        break;


                    }


                }


                break 'outer;


            }


        }


    }





    // 2. Find a prominent numeric element within the search area


    if let Some(area) = search_area {


        let mut candidate_elements = Vec::new();


        let selector = Selector::parse("*").map_err(|e| ScraperError(format!("{:?}", e)))?;





        for element in area.select(&selector) {


            let text = element.text().collect::<String>();


            let trimmed = text.trim().replace(",", ""); // Remove commas for parsing





            // Check if the text is a plausible price (contains digits, optionally commas/periods, and is not just the stock code)


            if !trimmed.is_empty() && trimmed.parse::<f64>().is_ok() && trimmed.len() > 2 && trimmed != anchor_text {


                 // Simple heuristic: consider tag name and class count for prominence


                let class_count = element.value().classes().count();


                candidate_elements.push((element, class_count));


            }


        }





        // Sort candidates by prominence (more classes = higher)


        candidate_elements.sort_by(|a, b| b.1.cmp(&a.1));





        if let Some((best_candidate, _)) = candidate_elements.first() {


            return Ok(Some(build_selector(best_candidate)));


        }


    }





    Ok(None)


}





async fn get_dow_dynamic_selectors(document: &Html) -> ScrapedSelectors {





    let mut scraped_selectors = ScrapedSelectors::default();





    let anchors = AnchorTexts::new();











    // Try to find name selector dynamically





    if let Ok(Some(selector)) = find_dynamic_selector(document, anchors.dow_name).await {





        scraped_selectors.name_selector = Some(selector);





    }











    // Try to find code selector dynamically





    if let Ok(Some(selector)) = find_dynamic_selector(document, anchors.dow_code).await {





        scraped_selectors.code_selector = Some(selector);





    }











    // Try to find price selector dynamically





    if let Ok(Some(selector)) = find_price_selector_near_anchor(document, anchors.dow_name, 4).await {





        scraped_selectors.price_selector = Some(selector);





    }











    // Try to find ratio selector dynamically





    if let Ok(Some(selector)) = find_ratio_selector_near_zenjitsuhi(document, anchors.zenjitsuhi, 4).await {





        scraped_selectors.ratio_selector = Some(selector);





    }











    // Try to find percent selector dynamically





    if let Ok(Some(selector)) = find_percent_selector_near_zenjitsuhi(document, anchors.zenjitsuhi, 4).await {





        scraped_selectors.percent_selector = Some(selector);





    }











    scraped_selectors





}





async fn get_stock_dynamic_selectors(document: &Html, known_name: &str) -> Result<ScrapedSelectors, Box<dyn Error>> {





    let mut scraped_selectors = ScrapedSelectors::default();





    let anchors = AnchorTexts::new();
    let zenjitsuhi_anchor = anchors.zenjitsuhi;











    // Try to find name selector using a specific H2 class


    let mut found_name_selector = None;


    let specific_h2_selector_str = "h2.PriceBoard__name__166W";


    if let Ok(selector) = Selector::parse(specific_h2_selector_str) {


        if let Some(element) = document.select(&selector).next() {


            let text = element.text().collect::<String>().trim().to_string();


            if !text.is_empty() && !text.chars().all(char::is_numeric) {


                found_name_selector = Some(build_selector(&element));


            }


        }


    }





    if found_name_selector.is_some() {


        scraped_selectors.name_selector = found_name_selector;


    } else if let Ok(Some(selector)) = find_dynamic_selector(document, known_name).await {


        scraped_selectors.name_selector = Some(selector);


    } else {


        // Fallback logic for name selector: iterate through h2 elements


        let h2_selector = Selector::parse("h2").map_err(|e| ScraperError(format!("{:?}", e)))?;


        for element in document.select(&h2_selector) {


            let text = element.text().collect::<String>().trim().to_string();


            // Check if the text is not empty and does not consist solely of numbers


            if !text.is_empty() && !text.chars().all(char::is_numeric) {


                found_name_selector = Some(build_selector(&element));


                break;


            }


        }


        if found_name_selector.is_some() {


            scraped_selectors.name_selector = found_name_selector;


        } else {


            // If still not found, fallback to a very general h2 selector


            // This should now be an error, not a fallback


            return Err(Box::new(ScraperError("Dynamic name selector not found for stock and no suitable h2 found.".to_string())));


        }


    }











    // Try to find code selector dynamically


    if known_name == "日経平均株価" {


        if let Ok(Some(selector)) = find_dynamic_selector(document, "998407.O").await {


            scraped_selectors.code_selector = Some(selector);


        }


    } else if let Ok(Some(selector)) = find_text_pattern_selector_near_anchor(document, known_name, "code").await {


        scraped_selectors.code_selector = Some(selector);


    }











    // Try to find price selector dynamically





    if let Ok(Some(selector)) = find_price_selector_near_anchor(document, known_name, 4).await {





        scraped_selectors.price_selector = Some(selector);





    }











    // Try to find ratio selector dynamically





    if let Ok(Some(selector)) = find_ratio_selector_near_zenjitsuhi(document, zenjitsuhi_anchor, 4).await {





        scraped_selectors.ratio_selector = Some(selector);





    }











    // Try to find percent selector dynamically





    if let Ok(Some(selector)) = find_percent_selector_near_zenjitsuhi(document, zenjitsuhi_anchor, 4).await {





        scraped_selectors.percent_selector = Some(selector);





    }











    Ok(scraped_selectors)





}





#[tokio::main]


async fn main() -> Result<(), Box<dyn Error>> {


    // --- Original main logic ---


    println!("--- Running Original Main Logic ---");


    /*


    let codes = r###"{


        "static_codes": ["%5EDJI", "7203", "6758"],


        "dynamic_codes": ["%5EDJI", "998407.O", "6758"]


    }"###;


    */


    let codes = r###"{


        "static_codes": [],


        "dynamic_codes": ["%5EDJI", "998407.O", "6758","8729","5016","4755","7203"]


    }"###;


    match fetch_data_rust(codes.to_string()).await {


        Ok(json_str) => {


            let scraped_data: Vec<StockData> = serde_json::from_str(&json_str)?;


            let (static_data, dynamic_data): (Vec<_>, Vec<_>) = scraped_data.into_iter().partition(|d| d.selector_type == "static");





            if !static_data.is_empty() {


                println!("\n--- Static Selector Results ---");


                for item in static_data {


                    println!("コード: {}", item.code);


                    println!("名前: {}", item.name);


                    println!("価格: {}", item.price);


                    println!("変化: {}", item.ratio);


                    println!("変化率: {}", item.percent);


                    println!("セレクタータイプ: {}", item.selector_type);


                    println!("---");


                }


            }





            if !dynamic_data.is_empty() {


                println!("\n--- Dynamic Selector Results ---");


                for item in dynamic_data {


                    println!("コード: {}", item.code);


                    println!("名前: {}", item.name);


                    println!("価格: {}", item.price);


                    println!("変化: {}", item.ratio);


                    println!("変化率: {}", item.percent);


                    println!("セレクタータイプ: {}", item.selector_type);


                    println!("---");


                }


            }


        }


        Err(err) => {


            eprintln!("エラーが発生しました: {:?}", err);


        }


    }


    println!("--- End of Original Main Logic ---\n");





    Ok(())


}





#[cfg(test)]


mod tests {


    use super::*;





    fn is_numeric_str(s: &str) -> bool {


        s.replace(",", "").parse::<f64>().is_ok()


    }





    #[tokio::test]


    async fn test_fetch_stock_sony() {


        let url = "https://finance.yahoo.co.jp/quote/6758.T";


        let result = crate::static_scraper::fetch_and_scrape_stock(url).await;


        assert!(result.is_ok());


        let data = result.unwrap();





        assert_eq!(data.code, "6758");


        assert_eq!(data.name, "ソニーグループ(株)");


        assert!(!data.price.is_empty());


        assert!(!data.ratio.is_empty());


        assert!(!data.percent.is_empty());





        assert!(is_numeric_str(&data.price));


        assert!(is_numeric_str(&data.ratio));


        assert!(is_numeric_str(&data.percent));


    }





    #[tokio::test]


    async fn test_fetch_dow() {


        let result = crate::static_scraper::fetch_and_scrape_dow().await;


        assert!(result.is_ok());


        let data = result.unwrap();





        assert_eq!(data.code, "^DJI");


        assert_eq!(data.name, "NYダウ");


        assert!(!data.price.is_empty());


        assert!(!data.ratio.is_empty());


        assert!(!data.percent.is_empty());





        assert!(is_numeric_str(&data.price));


        assert!(is_numeric_str(&data.ratio));


        assert!(is_numeric_str(&data.percent));


    }





    #[tokio::test]


    async fn test_fetch_nikkei() {


        let url = "https://finance.yahoo.co.jp/quote/998407.O";


        let result = crate::static_scraper::fetch_and_scrape_stock(url).await;


        assert!(result.is_ok());


        let data = result.unwrap();





        assert_eq!(data.code, "998407.O");


        assert_eq!(data.name, "日経平均株価");


        assert!(!data.price.is_empty());


        assert!(!data.ratio.is_empty());


        assert!(!data.percent.is_empty());





        assert!(is_numeric_str(&data.price));


        assert!(is_numeric_str(&data.ratio));


        assert!(is_numeric_str(&data.percent));


    }





    #[tokio::test]


    async fn test_fetch_data_rust_integration() {


        let codes = r###"{


            "static_codes": ["6758"],


            "dynamic_codes": ["7203"]


        }"###;


        let result = fetch_data_rust(codes.to_string()).await;


        assert!(result.is_ok());


        let json_str = result.unwrap();





        let scraped_data: Result<Vec<StockData>, _> = serde_json::from_str(&json_str);


        assert!(scraped_data.is_ok());


        let data_vec = scraped_data.unwrap();





        assert_eq!(data_vec.len(), 2);


    }


}





/// ヘルパー: ElementRefからセレクター文字列を生成


fn build_selector(element: &ElementRef) -> String {


    let tag_name = element.value().name();


    let classes = element.value().classes().collect::<Vec<_>>();


    if !classes.is_empty() {


        format!("{}.{}", tag_name, classes.join("."))


    } else {


        tag_name.to_string()


    }


}











