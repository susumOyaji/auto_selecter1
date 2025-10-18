use crate::models::{ScraperError};
use scraper::{ElementRef, Html, Selector};
use std::error::Error;

pub fn build_selector(element: &ElementRef) -> String {
    let tag_name = element.value().name();
    let classes = element.value().classes().collect::<Vec<_>>();
    if !classes.is_empty() {
        format!("{}.{}", tag_name, classes.join("."))
    } else {
        tag_name.to_string()
    }
}

pub async fn find_text_pattern_selector_near_anchor(
    document: &Html,
    anchor_text: &str,
    pattern_type: &str,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    const MAX_LEVELS: usize = 8; // Increased from 4
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

    if let Some(area) = search_area {
        for node in area.descendants() {
            if let Some(text_node) = node.value().as_text() {
                let trimmed_text = text_node.trim();
                let is_match = match pattern_type {
                    "code" => trimmed_text.len() == 4 && trimmed_text.chars().all(char::is_numeric),
                    "price" => {
                        trimmed_text.len() >= 4 && trimmed_text.chars().all(|c| c.is_numeric() || c == ',')
                    }
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

pub async fn find_percent_selector_near_zenjitsuhi(
    document: &Html,
    anchor_text: &str,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    const MAX_LEVELS: usize = 8;
    let mut search_area = None;
    'outer: for node in document.root_element().descendants() {
        if let Some(text_node) = node.value().as_text() {
            if text_node.trim() == anchor_text {
                let mut current = node.parent();
                for _ in 0..MAX_LEVELS {
                    if let Some(parent_node) = current {
                        if let Some(parent_element) = ElementRef::wrap(parent_node) {
                            search_area = Some(parent_element);
                            current = parent_node.parent();
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                break 'outer;
            }
        }
    }

    if let Some(area) = search_area {
        let selector = Selector::parse("*").map_err(|e| ScraperError(format!("{:?}", e)))?;
        let mut candidates = Vec::new();

        for element in area.select(&selector) {
            let text = element.text().collect::<String>();
            let trimmed = text.trim();

            if trimmed.contains('%')
                && trimmed.contains('(')
                && trimmed.contains(')')
                && trimmed.chars().any(|c| c.is_numeric())
                && !trimmed.contains("前日比")
            {
                candidates.push(element);
            }
        }

        let mut best_selector = None;
        for candidate in &candidates {
            let selector_str = build_selector(candidate);
            if selector_str.contains("secondary") {
                best_selector = Some(selector_str);
                break;
            }
        }

        if best_selector.is_some() {
            return Ok(best_selector);
        } else if let Some(fallback) = candidates.last() {
            let selector_str = build_selector(fallback);
            if selector_str.contains('.') {
                return Ok(Some(selector_str));
            }
        }
    }

    Ok(None)
}

pub async fn find_ratio_selector_near_zenjitsuhi(
    document: &Html,
    anchor_text: &str,
) -> Result<Option<String>, Box<dyn Error>> {
    const MAX_LEVELS: usize = 8; // Increased from 4
    let mut search_area = None;
    'outer: for node in document.root_element().descendants() {
        if let Some(text_node) = node.value().as_text() {
            if text_node.trim() == anchor_text {
                let mut current = node.parent();
                for _ in 0..MAX_LEVELS {
                    if let Some(parent_node) = current {
                        if let Some(parent_element) = ElementRef::wrap(parent_node) {
                            search_area = Some(parent_element);
                            current = parent_node.parent();
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                break 'outer;
            }
        }
    }

    if let Some(area) = search_area {
        let selector = Selector::parse("*").map_err(|e| ScraperError(format!("{:?}", e)))?;
        for element in area.select(&selector) {
            let text = element.text().collect::<String>();
            let trimmed = text.trim();

            if (trimmed.starts_with('+') || trimmed.starts_with('-'))
                && !trimmed.contains('%')
                && trimmed.len() > 1
            {
                let after_sign = &trimmed[1..].replace(",", "");
                if after_sign.parse::<f64>().is_ok() {
                    let selector_str = build_selector(&element);
                    if selector_str.contains('.') {
                        return Ok(Some(selector_str));
                    }
                }
            }
        }
    }

    Ok(None)
}

pub async fn find_price_selector_near_anchor(
    document: &Html,
    anchor_text: &str,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    const MAX_LEVELS: usize = 8; // Increased from 4
    let mut search_area = None;
    'outer: for node in document.root_element().descendants() {
        if let Some(text_node) = node.value().as_text() {
            if text_node.trim() == anchor_text {
                let mut current = node.parent();
                for _ in 0..MAX_LEVELS {
                    if let Some(parent_node) = current {
                        if let Some(parent_element) = ElementRef::wrap(parent_node) {
                            search_area = Some(parent_element);
                            current = parent_node.parent();
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                break 'outer;
            }
        }
    }

    if let Some(area) = search_area {
        let mut candidate_elements = Vec::new();
        let selector = Selector::parse("*").map_err(|e| ScraperError(format!("{:?}", e)))?;

        for element in area.select(&selector) {
            let text = element.text().collect::<String>();
            let trimmed_text = text.trim();

            // Skip values that start with + or - (likely a ratio)
            if trimmed_text.starts_with('+') || trimmed_text.starts_with('-') {
                continue;
            }

            let cleaned_text = trimmed_text.replace(",", "");
            let is_likely_stock_code = cleaned_text.len() == 4 && !cleaned_text.contains('.');

            if !cleaned_text.is_empty()
                && cleaned_text.parse::<f64>().is_ok()
                && !is_likely_stock_code
            {
                let class_count = element.value().classes().count();
                candidate_elements.push((element, class_count));
            }
        }

        candidate_elements.sort_by(|a, b| b.1.cmp(&a.1));

        if let Some((best_candidate, _)) = candidate_elements.first() {
            return Ok(Some(build_selector(best_candidate)));
        }
    }

    Ok(None)
}