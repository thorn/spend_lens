use std::time::Duration;
use regex::Regex;
use url::{Url};
use urlencoding;

const IIC_REGEX: &str = r"\.*iic\=([0-9a-fA-F]{32}).*";
const TIN_REGEX: &str = r".*tin\=(\d{8}).*";
const CRTD_REGEX: &str = r".*crtd\=(\d{4}-\d{2}-\d{2}T\d{2}:(\d{2}):(\d{2}(?:\.\d*)?)((-(\d{2}):(\d{2})|Z)?)).*";

#[derive(Debug)]
pub struct FiscalParams {
  pub iic: String,
  pub tin: String,
  pub crtd: String
}

pub fn download_check_info(fiscal_params: FiscalParams) -> Result<String, &'static str> {
  use reqwest::header::CONTENT_TYPE;
  let client = reqwest::blocking::Client::new();

  let form = reqwest::blocking::multipart::Form::new()
    .part("iic", reqwest::blocking::multipart::Part::text(fiscal_params.iic))
    .part("dateTimeCreated",reqwest::blocking::multipart::Part::text(fiscal_params.crtd))
    .part("tin",reqwest::blocking::multipart::Part::text(fiscal_params.tin));
  let res = client.post("https://mapr.tax.gov.me/ic/api/verifyInvoice")
    .header(CONTENT_TYPE, "multipart/form-data")
    .multipart(form)
    .timeout(Duration::from_secs(60))
    .send().expect("Request failed");

  return Ok(res.text().expect("Failed to retrieve the string"));
}

// Check that the URL is correct: `https://mapr.tax.gov.me` or `https://213.149.97.151`
// Check for the required parameters: iic, dateTimeCreated, and tin
// iic - fiscalization payer identification code, hexidecimal 32 bytes number
// crtd - date of purchase in format 2023-02-11T18:27:35+01:00
// tin - tax issuer number - 8-digit number, i.e. 02404281
//
// Note - Serbia has a similar system, but we don't support it yet
pub fn verify_invoice_url(check_url: &str) -> Result<(), &'static str>{
  // TODO: extract this to an upper level
  let check_url = urlencoding::decode(check_url).expect("UTF-8");

  let valid_domains = vec![
    String::from("https://mapr.tax.gov.me"),
    String::from("https://213.149.97.151"),
  ];

  let valid_serbian_domains = vec![
    String::from("https://suf.purs.gov.rs"),
  ];

  let parsed_url_result = Url::parse(&check_url);
  if parsed_url_result.is_err() { return Err("wrong_check_url"); }

  let parsed_url = parsed_url_result.unwrap();
  if !valid_domains.contains(&parsed_url.origin().unicode_serialization()) {
    if valid_serbian_domains.contains(&parsed_url.origin().unicode_serialization()) {
      return Err("serbian_checks_not_supported_yet");
    }
    return Err("wrong_check_url");
  }

  // TODO: this block repeats three times, it should be refactored
  let iic_regex = Regex::new(IIC_REGEX).unwrap();
  let caps = iic_regex.captures(&check_url);
  if caps.is_none() {
    return Err("iic_param_is_missing_or_wrong_format");
  }

  let date_time_created_regex = Regex::new(CRTD_REGEX).unwrap();
  let caps = date_time_created_regex.captures(&check_url);
  if caps.is_none() {
    return Err("crtd_param_is_missing_or_wrong_format");
  }

  let tin_regex = Regex::new(TIN_REGEX).unwrap();
  let caps = tin_regex.captures(&check_url);
  if caps.is_none() {
    return Err("tin_param_is_missing_or_wrong_format");
  }

  return Ok(());
}

pub fn extract_params_from_url(check_url: &str) -> Result<FiscalParams, &'static str> {
  // TODO: extract this to an upper level
  let check_url = urlencoding::decode(check_url).expect("UTF-8");

  let parsed_url = Url::parse(&check_url).map_err(|_| "URL parsing failed")?;
  let fragment = parsed_url.fragment().ok_or("Missing fragment")?;
  let fragment_params: Vec<&str> = fragment.split('?').collect();

  if fragment_params.len() < 2 {
    return Err("Missing parameters in fragment");
  }

  let params: Vec<(&str, &str)> = fragment_params[1].split('&')
    .filter_map(|part| {
      let mut split = part.splitn(2, '=');
      Some((split.next()?, split.next()?))
    })
    .collect();
  let params: std::collections::HashMap<&str, &str> = params.into_iter().collect();

  let iic = params.get("iic").ok_or("iic_param_is_missing_or_wrong_format")?.to_string();
  let crtd = params.get("crtd").ok_or("crtd_param_is_missing_or_wrong_format")?.replace("+", " ");
  let tin = params.get("tin").ok_or("tin_param_is_missing_or_wrong_format")?.to_string();

  Ok(FiscalParams {
    iic,
    crtd,
    tin,
  })
}

#[cfg(test)]
mod tests {
  use crate::{extract_params_from_url, verify_invoice_url};

  #[test]
  fn correct_invoice_mapr_domain_test() {
    let correct_check_url = "https://mapr.tax.gov.me/ic/#/verify/?iic=CDDDFEDA791C81615A66FFD8824ACFE0&tin=02404281&crtd=2023-02-11T18:27:35+01:00&ord=1973&bu=gt860be150&cr=wn519mv937&sw=ti937tv565&prc=514.00";
    assert!(verify_invoice_url(correct_check_url).is_ok());
  }
  #[test]
  fn correct_invoice_ip_address_test() {
    let correct_check_url = "https://213.149.97.151/ic/#/verify/?iic=CDDDFEDA791C81615A66FFD8824ACFE0&tin=02404281&crtd=2023-02-11T18:27:35+01:00&ord=1973&bu=gt860be150&cr=wn519mv937&sw=ti937tv565&prc=514.00";
    assert!(verify_invoice_url(correct_check_url).is_ok());
  }

  #[test]
  fn correct_invoice_urlencoded_test() {
    let encoded_url = "https://mapr.tax.gov.me/ic/#/verify?iic=569b2a25e33a44c5b755a5565dee180d&tin=03320758&crtd=2023-02-09T13%3A25%3A58%2B01%3A00&ord=59&bu=rf895ij778&cr=lx211ol284&sw=gg387fl042&prc=50.00";
    assert!(verify_invoice_url(encoded_url).is_ok());
  }

  #[test]
  fn malformed_url_test() {
    let invalid_check_url = "a_wrong_url/http";
    assert_eq!(verify_invoice_url(invalid_check_url), Err("wrong_check_url"));
  }

  #[test]
  fn serbian_invoice_url_test() {
    let serbian_check_url = "https://suf.purs.gov.rs/v/";
    assert_eq!(verify_invoice_url(serbian_check_url), Err("serbian_checks_not_supported_yet"));
  }

  #[test]
  fn extract_url_params_test() {
    let encoded_url = "https://mapr.tax.gov.me/ic/#/verify?iic=569b2a25e33a44c5b755a5565dee180d&tin=03320758&crtd=2023-02-09T13%3A25%3A58%2B01%3A00&ord=59&bu=rf895ij778&cr=lx211ol284&sw=gg387fl042&prc=50.00";
    let params = extract_params_from_url(encoded_url).unwrap();
    assert_eq!(params.iic, String::from("569b2a25e33a44c5b755a5565dee180d"));
    assert_eq!(params.tin, String::from("03320758"));
    assert_eq!(params.crtd, String::from("2023-02-09T13:25:58 01:00"));
  }
}