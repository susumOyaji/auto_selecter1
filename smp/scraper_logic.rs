use crate::models::{self, ScraperError};
use scraper::{ElementRef, Html, Selector};
use std::error::Error;

/// Finds a search area (an ancestor element) around a given text anchor.
fn find_search_area_around_anchor<'a>(document: &'a Html, anchor_text: &str) -> Option<ElementRef<'a>> {
    const MAX_LEVELS: usize = 8;
    let mut search_area = None;
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
    search_area
}

pub fn build_selector(element: &ElementRef) -> String {
    let tag_name = element.value().name();
    let classes = element.value().classes().collect::<Vec<_>>();
    if !classes.is_empty() {
        format!("{}.{}", tag_name, classes.join("."))
    } else {
        tag_name.to_string()
    }
}

/// Dynamically finds the name and its selector from the page.
pub async fn find_name_dynamically(document: &Html) -> Result<(Option<String>, String), Box<dyn Error>> {
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
            if text.contains("(株)") || text == "NYダウ" || text == "日経平均株価" || text.contains("/") {
                best_candidate_selector = Some(build_selector(&element));
                best_candidate_text = Some(text);
                break;
            }
            if fallback_candidate_selector.is_none() {
                fallback_candidate_selector = Some(build_selector(&element));
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
pub fn scrape_field(document: &Html, selector_opt: &Option<String>, _field_name: &str) -> String {
    if let Some(selector_str) = selector_opt {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(element) = document.select(&selector).next() {
                let value = element.text().collect::<String>().trim().to_string();
                return value;
            }
        }
    }
    String::new()
}

pub async fn find_text_pattern_selector_near_anchor(
    document: &Html,
    anchor_text: &str,
    pattern_type: &str,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    if let Some(area) = find_search_area_around_anchor(document, anchor_text) {
        for node in area.descendants() {
            if let Some(text_node) = node.value().as_text() {
                let trimmed_text = text_node.trim();
                let is_match = match pattern_type {
                    "code" => trimmed_text.len() == 4 && trimmed_text.chars().all(char::is_numeric),
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

// --- Stock-specific finders (using "前日比" anchor) ---

pub async fn find_stock_price_selector(
    document: &Html,
    anchor_text: &str,
    code: &str, // New parameter to avoid mistaking the code for the price
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    if let Some(name_area) = find_search_area_around_anchor(document, anchor_text) {
        let mut zenjitsuhi_element_opt = None;
        let zenjitsuhi_selector = Selector::parse("*").map_err(|e| ScraperError(format!("{:?}", e)))?;
        for element in name_area.select(&zenjitsuhi_selector) {
            if element.text().collect::<String>().trim() == "前日比" {
                zenjitsuhi_element_opt = Some(element);
                break;
            }
        }

        if let Some(zenjitsuhi_element) = zenjitsuhi_element_opt {
            let mut current_element = zenjitsuhi_element;
            loop {
                for sibling in current_element.prev_siblings() {
                    if let Some(sibling_element) = ElementRef::wrap(sibling) {
                        let span_selector = Selector::parse("span").map_err(|e| ScraperError(format!("{:?}", e)))?;
                        for span_element in sibling_element.select(&span_selector) {
                            let text = span_element.text().collect::<String>();
                            let trimmed_text = text.trim();
                            let cleaned_text = trimmed_text.replace(",", "");

                            if !cleaned_text.is_empty()
                                && cleaned_text.parse::<f64>().is_ok()
                                && !trimmed_text.starts_with('+')
                                && !trimmed_text.starts_with('-')
                                && !trimmed_text.contains('%')
                                && cleaned_text != code // <-- The key fix
                            {
                                return Ok(Some(build_selector(&span_element)));
                            }
                        }
                    }
                }

                if let Some(parent) = current_element.parent().and_then(ElementRef::wrap) {
                    current_element = parent;
                } else {
                    break; // No more parents to check
                }
            }
        }
    }

    Ok(None)
}

pub async fn find_stock_change_selector(
    document: &Html,
    anchor_text: &str,
) -> Result<Option<String>, Box<dyn Error>> {
    if let Some(area) = find_search_area_around_anchor(document, anchor_text) {
        let selector = Selector::parse("*").map_err(|e| ScraperError(format!("{:?}", e)))?;
        for element in area.select(&selector) {
            let text = element.text().collect::<String>();
            let trimmed = text.trim();

            if (trimmed.starts_with('+') || trimmed.starts_with('-')) && !trimmed.contains('%') && trimmed.len() > 1 {
                let after_sign = &trimmed[1..].replace(",", "");
                if after_sign.parse::<f64>().is_ok() {
                    return Ok(Some(build_selector(&element)));
                }
            }
        }
    }
    Ok(None)
}

pub async fn find_stock_change_percent_selector(
    document: &Html,
    anchor_text: &str,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    if let Some(area) = find_search_area_around_anchor(document, anchor_text) {
        let span_selector = Selector::parse("span").map_err(|e| ScraperError(format!("{:?}", e)))?;
        for span_element in area.select(&span_selector) {
            let text = span_element.text().collect::<String>();
            let trimmed = text.trim();

            if trimmed.starts_with('(')
                && trimmed.ends_with(')')
                && trimmed.contains('%')
                && trimmed.chars().any(|c| c.is_numeric())
            {
                return Ok(Some(build_selector(&span_element)));
            }
        }
    }
    Ok(None)
}

pub async fn find_stock_update_time_selector(
    document: &Html,
) -> Result<Option<String>, Box<dyn Error>> {
    if let Some(area) = find_search_area_around_anchor(document, "リアルタイム株価") {
        let footer_selector = Selector::parse("*").map_err(|e| ScraperError(format!("{:?}", e)))?;
        if let Some(footer_element) = area.select(&footer_selector).find(|element| {
            if let Some(class) = element.value().attr("class") {
                class.contains("PriceBoard__mainFooter")
            } else {
                false
            }
        }) {
            let time_tag_selector = Selector::parse("time").map_err(|e| ScraperError(format!("{:?}", e)))?;
            if let Some(time_element) = footer_element.select(&time_tag_selector).next() {
                return Ok(Some(build_selector(&time_element)));
            }
        }
    }

    Ok(None)
}

// --- Index-specific finders ---
pub async fn find_dji_update_time_selector(
    document: &Html,
) -> Result<Option<String>, Box<dyn Error>> {
    // Find the footer element which seems to have a stable class name, based on user's provided selector.
    let footer_selector_str = "._CommonPriceBoard__mainFooter_1g7gt_48";
    let footer_selector = Selector::parse(footer_selector_str)
        .map_err(|e| ScraperError(format!("Failed to parse index footer selector: {:?}", e)))?;

    if let Some(footer_element) = document.select(&footer_selector).next() {
        // Within that footer, find the <time> element.
        let time_selector = Selector::parse("time")
            .map_err(|e| ScraperError(format!("Failed to parse time tag selector: {:?}", e)))?;
        if let Some(time_element) = footer_element.select(&time_selector).next() {
            return Ok(Some(build_selector(&time_element)));
        }
    }

    Ok(None)
}

pub async fn find_nikkei_update_time_selector(
    document: &Html,
) -> Result<Option<String>, Box<dyn Error>> {
    // Find the footer element which seems to have a stable class name, based on user's provided selector.
    let footer_selector_str = ".PriceBoard__mainFooter__16pO";
    let footer_selector = Selector::parse(footer_selector_str)
        .map_err(|e| ScraperError(format!("Failed to parse Nikkei footer selector: {:?}", e)))?;

    if let Some(footer_element) = document.select(&footer_selector).next() {
        // Within that footer, find the <time> element.
        let time_selector = Selector::parse("time")
            .map_err(|e| ScraperError(format!("Failed to parse time tag selector: {:?}", e)))?;
        if let Some(time_element) = footer_element.select(&time_selector).next() {
            return Ok(Some(build_selector(&time_element)));
        }
    }

    Ok(None)
}

// --- FX-specific finders (using "Bid", "Change" anchors) ---

pub async fn find_fx_price_selector(
    document: &Html,
) -> Result<Option<String>, Box<dyn Error>> {
    if let Some(area) = find_search_area_around_anchor(document, "Bid") {
        let span_selector = Selector::parse("span").map_err(|e| ScraperError(format!("{:?}", e)))?;
        for span_element in area.select(&span_selector) {
            let text = span_element.text().collect::<String>();
            let trimmed_text = text.trim();
            let cleaned_text = trimmed_text.replace(",", "");

            if !cleaned_text.is_empty() && cleaned_text.parse::<f64>().is_ok() {
                return Ok(Some(build_selector(&span_element)));
            }
        }
    }
    Ok(None)
}

pub async fn find_fx_change_selector(
    document: &Html,
) -> Result<Option<String>, Box<dyn Error>> {
    if let Some(area) = find_search_area_around_anchor(document, "Change") {
        let span_selector = Selector::parse("span").map_err(|e| ScraperError(format!("{:?}", e)))?;
        for span_element in area.select(&span_selector) {
            let text = span_element.text().collect::<String>();
            let trimmed = text.trim();

            if (trimmed.starts_with('+') || trimmed.starts_with('-')) && !trimmed.contains('%') && trimmed.len() > 1 {
                let after_sign = &trimmed[1..].replace(",", "");
                if after_sign.parse::<f64>().is_ok() {
                    return Ok(Some(build_selector(&span_element)));
                }
            }
        }
    }
    Ok(None)
}

pub async fn find_fx_update_time_selector(
    document: &Html,
) -> Result<Option<String>, Box<dyn Error>> {
    if let Some(area) = find_search_area_around_anchor(document, "Bid") {
        let span_selector = Selector::parse("span").map_err(|e| ScraperError(format!("{:?}", e)))?;
        for span_element in area.select(&span_selector) {
            let text = span_element.text().collect::<String>();
            let trimmed = text.trim();

            if trimmed.contains(':') && trimmed.contains('(') && trimmed.contains(')') && trimmed.len() < 20 {
                return Ok(Some(build_selector(&span_element)));
            }
        }
    }
    Ok(None)
}
