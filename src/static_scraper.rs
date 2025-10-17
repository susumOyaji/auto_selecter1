use crate::{StockData, ScraperError};
use reqwest;
use scraper::{Html, Selector};
use std::error::Error;

pub async fn scrape_statically(code: &str) -> Result<StockData, Box<dyn Error>> {
    match code {
        "%5EDJI" => fetch_and_scrape_dow().await,
        _ => {
            let url = if code == "998407.O" {
                format!("https://finance.yahoo.co.jp/quote/{}", code)
            } else {
                format!("https://finance.yahoo.co.jp/quote/{}.T", code)
            };
            fetch_and_scrape_stock(&url).await
        }
    }
}

pub async fn fetch_and_scrape_stock(url: &str) -> Result<StockData, Box<dyn std::error::Error>> {
    let response = reqwest::get(url).await?;
    let body = response.text().await?;
    let document = Html::parse_document(&body);

    let code_selector = Selector::parse("span.PriceBoard__code__SnMF").map_err(|e| ScraperError(format!("{:?}", e)))?;
    let name_selector = Selector::parse("h2.PriceBoard__name__166W").map_err(|e| ScraperError(format!("{:?}", e)))?;
    let price_selector = Selector::parse("span.StyledNumber__value__3rXW").map_err(|e| ScraperError(format!("{:?}", e)))?;
    let ratio_selector = Selector::parse("dd.PriceChangeLabel__description__a5Lp > span.StyledNumber__1fof > span.PriceChangeLabel__primary__Y_ut > span.StyledNumber__value__3rXW").map_err(|e| ScraperError(format!("{:?}", e)))?;
    let percent_selector = Selector::parse("dd.PriceChangeLabel__description__a5Lp > span.StyledNumber__1fof > span.StyledNumber__item--secondary__RTJc > span.StyledNumber__value__3rXW").map_err(|e| ScraperError(format!("{:?}", e)))?;

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
        selector_type: "static".to_string(),
    })
}

pub async fn fetch_and_scrape_dow() -> Result<StockData, Box<dyn Error>> {
    let url = "https://finance.yahoo.co.jp/quote/%5EDJI"; // NYダウ平均のURL
    let response = reqwest::get(url).await?;
    let body = response.text().await?;
    let document = Html::parse_document(&body);

    let code_selector = Selector::parse("span._CommonPriceBoard__code_1g7gt_11").map_err(|e| ScraperError(format!("{:?}", e)))?;
    let name_selector = Selector::parse("h2._BasePriceBoard__name_1tkwp_66").map_err(|e| ScraperError(format!("{:?}", e)))?;
    let price_selector = Selector::parse("span._StyledNumber__value_1lush_9").map_err(|e| ScraperError(format!("{:?}", e)))?;
    let ratio_selector = Selector::parse("span._PriceChangeLabel__primary_hse06_56 > span._StyledNumber__value_1lush_9").map_err(|e| ScraperError(format!("{:?}", e)))?;
    let percent_selector = Selector::parse("span._PriceChangeLabel__secondary_hse06_62 > span._StyledNumber__value_1lush_9").map_err(|e| ScraperError(format!("{:?}", e)))?;

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
        selector_type: "static".to_string(),
    })
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
        let result = fetch_and_scrape_stock(url).await;
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
        let result = fetch_and_scrape_dow().await;
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
        let result = fetch_and_scrape_stock(url).await;
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
}