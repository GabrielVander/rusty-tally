// //! A parser for OFX 1.0.2 files, built using Test-Driven Development.
// //! To see log output, run with `RUST_LOG=info cargo run`.
// use log::{debug, error, info, warn};
// use std::collections::HashMap;
//
// #[derive(Debug, PartialEq)]
// pub enum ParseError {
//     MissingHeader,
//     InvalidHeader(String),
// }
//
// #[derive(Debug, PartialEq, Clone)]
// pub struct Header {
//     pub ofx_header: String,
//     pub data: String,
//     pub version: String,
//     pub security: String,
//     pub encoding: String,
//     pub charset: String,
//     pub compression: String,
//     pub old_file_uid: String,
//     pub new_file_uid: String,
// }
//
// #[derive(Debug, Clone, PartialEq)]
// pub struct SignonInfo {
//     pub status_code: String,
//     pub status_severity: String,
//     pub dt_server: String,
//     pub language: String,
// }
//
// #[derive(Debug, Clone, PartialEq)]
// pub struct BankAccount {
//     pub trx_uid: String,
//     pub status_code: String,
//     pub status_severity: String,
//     pub statement: BankStatement,
//     pub acct_type: String,
//     pub acct_id: String,
//     pub bank_id: String,
// }
//
// #[derive(Debug, Clone, PartialEq)]
// pub struct BankStatement {
//     pub currency: String,
//     pub start_date: String,
//     pub end_date: String,
//     pub transactions: Vec<Transaction>,
// }
//
// #[derive(Debug, Clone, PartialEq)]
// pub struct Transaction {
//     pub trx_type: String,
//     pub dt_posted: String,
//     pub trx_amount: String,
//     pub fit_id: String,
//     pub name: String,
//     pub memo: String,
// }
//
// #[derive(Debug, PartialEq)]
// pub struct Ofx {
//     pub header: Header,
//     pub signon_info: Option<SignonInfo>,
//     pub bank_account: Option<BankAccount>,
// }
//
// // Represents the current section being parsed within the OFX body.
// #[derive(Debug, Clone, Copy)]
// enum ParserState {
//     None,
//     InSignon,
//     InBank,
//     InTransaction,
// }
//
// impl Ofx {
//     pub fn from_lines<I, S>(mut lines: I) -> Result<Ofx, ParseError>
//     where
//         I: Iterator<Item = S>,
//         S: AsRef<str>,
//     {
//         info!("Starting OFX parsing process.");
//         let mut headers_map: HashMap<String, String> = HashMap::new();
//         let mut found_ofx_tag: bool = false;
//
//         debug!("Parsing headers...");
//         for line_ref in &mut lines {
//             let line = line_ref.as_ref().trim();
//             if line.is_empty() {
//                 continue;
//             }
//             if line.starts_with("<OFX>") {
//                 info!("Found <OFX> tag, header parsing complete.");
//                 found_ofx_tag = true;
//                 break;
//             }
//             let parts: Vec<&str> = line.splitn(2, ':').collect();
//             if parts.len() == 2 {
//                 debug!(
//                     "Parsed header: '{}' -> '{}'",
//                     parts[0].trim(),
//                     parts[1].trim()
//                 );
//                 headers_map.insert(parts[0].trim().to_string(), parts[1].trim().to_string());
//             } else {
//                 error!("Malformed header line found: {}", line);
//                 return Err(ParseError::InvalidHeader(format!(
//                     "Malformed header line: {}",
//                     line
//                 )));
//             }
//         }
//
//         if !found_ofx_tag {
//             error!("Parsing failed: <OFX> tag not found, file is missing body.");
//             return Err(ParseError::MissingHeader);
//         }
//
//         let header = Header {
//             ofx_header: headers_map
//                 .get("OFXHEADER")
//                 .cloned()
//                 .ok_or_else(|| ParseError::InvalidHeader("Missing OFXHEADER".to_string()))?,
//             data: headers_map
//                 .get("DATA")
//                 .cloned()
//                 .ok_or_else(|| ParseError::InvalidHeader("Missing DATA".to_string()))?,
//             version: headers_map
//                 .get("VERSION")
//                 .cloned()
//                 .ok_or_else(|| ParseError::InvalidHeader("Missing VERSION".to_string()))?,
//             security: headers_map
//                 .get("SECURITY")
//                 .cloned()
//                 .ok_or_else(|| ParseError::InvalidHeader("Missing SECURITY".to_string()))?,
//             encoding: headers_map
//                 .get("ENCODING")
//                 .cloned()
//                 .ok_or_else(|| ParseError::InvalidHeader("Missing ENCODING".to_string()))?,
//             charset: headers_map
//                 .get("CHARSET")
//                 .cloned()
//                 .ok_or_else(|| ParseError::InvalidHeader("Missing CHARSET".to_string()))?,
//             compression: headers_map
//                 .get("COMPRESSION")
//                 .cloned()
//                 .ok_or_else(|| ParseError::InvalidHeader("Missing COMPRESSION".to_string()))?,
//             old_file_uid: headers_map
//                 .get("OLDFILEUID")
//                 .cloned()
//                 .ok_or_else(|| ParseError::InvalidHeader("Missing OLDFILEUID".to_string()))?,
//             new_file_uid: headers_map
//                 .get("NEWFILEUID")
//                 .cloned()
//                 .ok_or_else(|| ParseError::InvalidHeader("Missing NEWFILEUID".to_string()))?,
//         };
//         info!("Header parsed successfully.");
//         debug!("Validating header fields...");
//
//         if header.version != "102" {
//             error!(
//                 "Header validation failed: Unsupported OFX version '{}'",
//                 header.version
//             );
//             return Err(ParseError::InvalidHeader(format!(
//                 "Unsupported OFX version: {}",
//                 header.version
//             )));
//         }
//         if header.data != "OFXSGML" {
//             error!(
//                 "Header validation failed: Unsupported data type '{}'",
//                 header.data
//             );
//             return Err(ParseError::InvalidHeader(format!(
//                 "Unsupported data type: {}",
//                 header.data
//             )));
//         }
//         info!("Header validation successful.");
//
//         // --- Body Parsing ---
//         info!("Parsing OFX body...");
//         let mut signon_info = None;
//         let mut bank_account = None;
//
//         let mut state = ParserState::None;
//         let mut current_tag_content: HashMap<String, String> = HashMap::new();
//         let mut current_transaction_content: HashMap<String, String> = HashMap::new();
//         let mut transactions: Vec<Transaction> = Vec::new();
//
//         for line_ref in lines {
//             let line = line_ref.as_ref().trim();
//             if line.is_empty() {
//                 continue;
//             }
//
//             if line.starts_with('<') && line.ends_with('>') {
//                 let tag = line.trim_matches(|c| c == '<' || c == '>');
//
//                 if tag.starts_with('/') {
//                     // Closing tag
//                     let tag_name = tag.trim_start_matches('/');
//                     debug!("Encountered closing tag: </{}>", tag_name);
//                     match tag_name {
//                         "SIGNONMSGSRSV1" => {
//                             signon_info = Some(SignonInfo {
//                                 status_code: current_tag_content
//                                     .get("CODE")
//                                     .cloned()
//                                     .unwrap_or_default(),
//                                 status_severity: current_tag_content
//                                     .get("SEVERITY")
//                                     .cloned()
//                                     .unwrap_or_default(),
//                                 dt_server: current_tag_content
//                                     .get("DTSERVER")
//                                     .cloned()
//                                     .unwrap_or_default(),
//                                 language: current_tag_content
//                                     .get("LANGUAGE")
//                                     .cloned()
//                                     .unwrap_or_default(),
//                             });
//                             info!("Finished parsing Sign-on block (SIGNONMSGSRSV1).");
//                             current_tag_content.clear();
//                             state = ParserState::None;
//                         }
//                         "BANKMSGSRSV1" => {
//                             let statement = BankStatement {
//                                 currency: current_tag_content
//                                     .get("CURDEF")
//                                     .cloned()
//                                     .unwrap_or_default(),
//                                 start_date: current_tag_content
//                                     .get("DTSTART")
//                                     .cloned()
//                                     .unwrap_or_default(),
//                                 end_date: current_tag_content
//                                     .get("DTEND")
//                                     .cloned()
//                                     .unwrap_or_default(),
//                                 transactions: transactions.clone(),
//                             };
//                             bank_account = Some(BankAccount {
//                                 trx_uid: current_tag_content
//                                     .get("TRNUID")
//                                     .cloned()
//                                     .unwrap_or_default(),
//                                 status_code: current_tag_content
//                                     .get("CODE")
//                                     .cloned()
//                                     .unwrap_or_default(),
//                                 status_severity: current_tag_content
//                                     .get("SEVERITY")
//                                     .cloned()
//                                     .unwrap_or_default(),
//                                 statement,
//                                 acct_type: current_tag_content
//                                     .get("ACCTTYPE")
//                                     .cloned()
//                                     .unwrap_or_default(),
//                                 acct_id: current_tag_content
//                                     .get("ACCTID")
//                                     .cloned()
//                                     .unwrap_or_default(),
//                                 bank_id: current_tag_content
//                                     .get("BANKID")
//                                     .cloned()
//                                     .unwrap_or_default(),
//                             });
//                             info!("Finished parsing Bank Statement block (BANKMSGSRSV1).");
//                             state = ParserState::None;
//                         }
//                         "STMTTRN" => {
//                             if let ParserState::InTransaction = state {
//                                 transactions.push(Transaction {
//                                     trx_type: current_transaction_content
//                                         .get("TRNTYPE")
//                                         .cloned()
//                                         .unwrap_or_default(),
//                                     dt_posted: current_transaction_content
//                                         .get("DTPOSTED")
//                                         .cloned()
//                                         .unwrap_or_default(),
//                                     trx_amount: current_transaction_content
//                                         .get("TRNAMT")
//                                         .cloned()
//                                         .unwrap_or_default(),
//                                     fit_id: current_transaction_content
//                                         .get("FITID")
//                                         .cloned()
//                                         .unwrap_or_default(),
//                                     name: current_transaction_content
//                                         .get("NAME")
//                                         .cloned()
//                                         .unwrap_or_default(),
//                                     memo: current_transaction_content
//                                         .get("MEMO")
//                                         .cloned()
//                                         .unwrap_or_default(),
//                                 });
//                                 debug!("Parsed transaction: {:?}", transactions.last().unwrap());
//                                 current_transaction_content.clear();
//                                 state = ParserState::InBank;
//                             }
//                         }
//                         _ => {} // Ignore other closing tags for now
//                     }
//                 } else {
//                     // Opening tag
//                     debug!("Encountered opening tag: <{}>", tag);
//                     match tag {
//                         "SIGNONMSGSRSV1" => state = ParserState::InSignon,
//                         "BANKMSGSRSV1" => state = ParserState::InBank,
//                         "STMTTRN" => state = ParserState::InTransaction,
//                         _ => {} // Ignore other opening tags
//                     }
//                     debug!("Parser state changed to: {:?}", state);
//                 }
//             } else {
//                 // Data line
//                 if let Some(open_tag_end) = line.find('>') {
//                     let open_tag = &line[1..open_tag_end];
//                     let rest_of_line = &line[open_tag_end + 1..];
//
//                     // Check for an inline closing tag and extract the value between them.
//                     let closing_tag_str = format!("</{}>", open_tag);
//                     let value = if let Some(value_end_pos) = rest_of_line.find(&closing_tag_str) {
//                         &rest_of_line[..value_end_pos]
//                     } else {
//                         // If no closing tag on the same line, the value is the rest of the line.
//                         rest_of_line
//                     };
//
//                     debug!("Found data for tag <{}>: '{}'", open_tag, value);
//                     match state {
//                         ParserState::InTransaction => {
//                             current_transaction_content
//                                 .insert(open_tag.to_string(), value.to_string());
//                         }
//                         ParserState::InSignon | ParserState::InBank => {
//                             current_tag_content.insert(open_tag.to_string(), value.to_string());
//                         }
//                         ParserState::None => {
//                             warn!("Found data line outside of a known section: {}", line);
//                         }
//                     }
//                 }
//             }
//         }
//
//         info!("OFX parsing process completed successfully.");
//         Ok(Ofx {
//             header,
//             signon_info,
//             bank_account,
//         })
//     }
// }
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     fn get_valid_header() -> Header {
//         Header {
//             ofx_header: "100".to_string(),
//             data: "OFXSGML".to_string(),
//             version: "102".to_string(),
//             security: "NONE".to_string(),
//             encoding: "USASCII".to_string(),
//             charset: "1252".to_string(),
//             compression: "NONE".to_string(),
//             old_file_uid: "NONE".to_string(),
//             new_file_uid: "NONE".to_string(),
//         }
//     }
//
//     fn get_valid_ofx_lines() -> Vec<&'static str> {
//         vec![
//             "OFXHEADER:100",
//             "DATA:OFXSGML",
//             "VERSION:102",
//             "SECURITY:NONE",
//             "ENCODING:USASCII",
//             "CHARSET:1252",
//             "COMPRESSION:NONE",
//             "OLDFILEUID:NONE",
//             "NEWFILEUID:NONE",
//             "",
//             "<OFX>",
//             "<SIGNONMSGSRSV1>",
//             "<SONRS>",
//             "<STATUS>",
//             "<CODE>0",
//             "<SEVERITY>INFO",
//             "</STATUS>",
//             "<DTSERVER>20250720160000",
//             "<LANGUAGE>ENG",
//             "</SONRS>",
//             "</SIGNONMSGSRSV1>",
//             "<BANKMSGSRSV1>",
//             "<STMTTRNRS>",
//             "<TRNUID>1",
//             "<STATUS>",
//             "<CODE>0",
//             "<SEVERITY>INFO",
//             "</STATUS>",
//             "<STMTRS>",
//             "<CURDEF>USD",
//             "<BANKACCTFROM>",
//             "<BANKID>12345",
//             "<ACCTID>54321",
//             "<ACCTTYPE>CHECKING",
//             "</BANKACCTFROM>",
//             "<BANKTRANLIST>",
//             "<DTSTART>20250701",
//             "<DTEND>20250720",
//             "<STMTTRN>",
//             "<TRNTYPE>DEBIT",
//             "<DTPOSTED>20250715",
//             "<TRNAMT>-100.00",
//             "<FITID>123",
//             "<NAME>Coffee Shop",
//             "<MEMO>Morning coffee",
//             "</STMTTRN>",
//             "<STMTTRN>",
//             "<TRNTYPE>CREDIT",
//             "<DTPOSTED>20250718",
//             "<TRNAMT>2000.00",
//             "<FITID>456",
//             "<NAME>Salary",
//             "<MEMO>Paycheck",
//             "</STMTTRN>",
//             "</BANKTRANLIST>",
//             "</STMTRS>",
//             "</STMTTRNRS>",
//             "</BANKMSGSRSV1>",
//             "</OFX>",
//         ]
//     }
//
//     #[test]
//     fn test_empty_input_fails() {
//         let lines: Vec<&str> = vec![];
//         let result = Ofx::from_lines(lines.into_iter());
//         assert!(result.is_err());
//         match result.err().unwrap() {
//             ParseError::MissingHeader => {} // Expected
//             e => panic!("Expected MissingHeader error, got {:?}", e),
//         }
//     }
//
//     #[test]
//     fn test_parses_valid_header() {
//         let lines = get_valid_ofx_lines();
//         let result = Ofx::from_lines(lines.into_iter());
//         assert!(result.is_ok());
//         let ofx = result.unwrap();
//         assert_eq!(ofx.header, get_valid_header());
//     }
//
//     #[test]
//     fn test_fails_on_unsupported_version() {
//         let mut lines = get_valid_ofx_lines();
//         lines[2] = "VERSION:200"; // Change to an unsupported version
//         let result = Ofx::from_lines(lines.into_iter());
//         assert!(result.is_err());
//         match result.err().unwrap() {
//             ParseError::InvalidHeader(msg) => {
//                 assert!(msg.contains("Unsupported OFX version"));
//             }
//             e => panic!("Expected InvalidHeader error, got {:?}", e),
//         }
//     }
//
//     #[test]
//     fn test_fails_on_unsupported_data_type() {
//         let mut lines = get_valid_ofx_lines();
//         lines[1] = "DATA:XML"; // Change to an unsupported data type
//         let result = Ofx::from_lines(lines.into_iter());
//         assert!(result.is_err());
//         match result.err().unwrap() {
//             ParseError::InvalidHeader(msg) => {
//                 assert!(msg.contains("Unsupported data type"));
//             }
//             e => panic!("Expected InvalidHeader error, got {:?}", e),
//         }
//     }
//
//     #[test]
//     fn test_parses_signon_message() {
//         let lines = get_valid_ofx_lines();
//         let result = Ofx::from_lines(lines.into_iter());
//         assert!(result.is_ok());
//         let ofx = result.unwrap();
//         assert!(ofx.signon_info.is_some());
//         let signon = ofx.signon_info.unwrap();
//         assert_eq!(signon.status_code, "0");
//         assert_eq!(signon.status_severity, "INFO");
//         assert_eq!(signon.dt_server, "20250720160000");
//         assert_eq!(signon.language, "ENG");
//     }
//
//     #[test]
//     fn test_parses_bank_account_and_transactions() {
//         let lines = get_valid_ofx_lines();
//         let result = Ofx::from_lines(lines.into_iter());
//         assert!(result.is_ok());
//         let ofx = result.unwrap();
//
//         assert!(ofx.bank_account.is_some());
//         let account = ofx.bank_account.unwrap();
//         assert_eq!(account.bank_id, "12345");
//         assert_eq!(account.acct_id, "54321");
//         assert_eq!(account.acct_type, "CHECKING");
//
//         let statement = account.statement;
//         assert_eq!(statement.currency, "USD");
//         assert_eq!(statement.start_date, "20250701");
//         assert_eq!(statement.end_date, "20250720");
//         assert_eq!(statement.transactions.len(), 2);
//
//         let t1 = &statement.transactions[0];
//         assert_eq!(t1.trx_type, "DEBIT");
//         assert_eq!(t1.trx_amount, "-100.00");
//         assert_eq!(t1.name, "Coffee Shop");
//
//         let t2 = &statement.transactions[1];
//         assert_eq!(t2.trx_type, "CREDIT");
//         assert_eq!(t2.trx_amount, "2000.00");
//         assert_eq!(t2.name, "Salary");
//     }
//
//     #[test]
//     fn test_parses_inline_tags_correctly() {
//         let mut lines = get_valid_ofx_lines();
//         // Replace a multi-line tag with a single-line one
//         lines[46] = "<NAME>Inline Coffee Shop</NAME>";
//
//         let result = Ofx::from_lines(lines.into_iter());
//         assert!(result.is_ok(), "Parsing failed: {:?}", result.err());
//         let ofx = result.unwrap();
//
//         let account = ofx.bank_account.expect("Bank account should be present");
//         let t1 = &account.statement.transactions[0];
//         assert_eq!(t1.name, "Inline Coffee Shop");
//     }
// }

use std::str::FromStr;

// src/domain/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OfxError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("XML parsing error: {0}")]
    Xml(#[from] quick_xml::DeError),

    #[error("Invalid OFX version: {0}")]
    InvalidVersion(String),

    #[error("Missing required header: {0}")]
    MissingHeader(String),

    #[error("Invalid date format: {0}")]
    InvalidDateFormat(String),

    #[error("Invalid content: {0}")]
    InvalidContent(String),

    #[error("Unsupported OFX feature: {0}")]
    UnsupportedFeature(String),
}

pub type OfxResult<T> = Result<T, OfxError>;

// src/domain/models.rs
use chrono::{DateTime, FixedOffset, NaiveDateTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OfxHeader {
    pub version: String,
    pub security: Option<String>,
    pub encoding: Option<String>,
    pub charset: Option<String>,
    pub compression: Option<String>,
    pub old_file_uid: Option<String>,
    pub new_file_uid: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SignonMessage {
    pub sonrs: SignonResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SignonResponse {
    pub status: Status,
    pub dtserver: DateTime<FixedOffset>,
    pub language: Option<String>,
    pub dtprofup: Option<DateTime<FixedOffset>>,
    pub fi: Option<FinancialInstitution>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Status {
    pub code: i32,
    pub severity: String,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FinancialInstitution {
    pub org: String,
    pub fid: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BankAccount {
    pub bankid: String,
    pub acctid: String,
    pub accttype: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    pub trntype: String,
    pub dtposted: DateTime<FixedOffset>,
    pub trnamt: f64,
    pub fitid: String,
    pub name: Option<String>,
    pub memo: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BankTransactionList {
    pub dtstart: DateTime<FixedOffset>,
    pub dtend: DateTime<FixedOffset>,
    pub transactions: Vec<Transaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StatementTransactionResponse {
    pub trnuid: String,
    pub status: Status,
    pub stmtrs: StatementResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StatementResponse {
    pub curdef: String,
    pub bankacctfrom: BankAccount,
    pub banktranlist: Option<BankTransactionList>,
    pub ledgerbal: Option<Balance>,
    pub availbal: Option<Balance>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Balance {
    pub balamt: f64,
    pub dtasof: DateTime<FixedOffset>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OfxDocument {
    pub header: OfxHeader,
    pub signon: SignonResponse,
    pub bank_msgs: Vec<StatementTransactionResponse>,
}

use log::{debug, error, info, warn};
use quick_xml::de::from_str;
use regex::Regex;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
struct OfxXml {
    signonmsgsrsv1: SignOnMessageResponseV1Xml,
    bankmsgsrsv1: BankMessageResponseV1Xml,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
struct SignOnMessageResponseV1Xml {
    sonrs: SignOnResponseXml,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
struct BankMessageResponseV1Xml {
    stmttrnrs: Vec<StatementTransactionResponseXml>, // Matches the XML structure for multiple STMTTRNRS elements
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
struct SignOnResponseXml {
    status: StatusXml,
    dtserver: String,
    language: Option<String>,
    dtprofup: Option<String>,
    fi: Option<FinancialInstitutionXml>,
}

impl Into<SignonResponse> for SignOnResponseXml {
    fn into(self) -> SignonResponse {
        SignonResponse {
            status: self.status.into(),
            dtserver: OfxParser::parse_custom_datetime(&self.dtserver)
                .inspect_err(|e: &String| error!("{e}"))
                .unwrap_or_default(),
            language: self.language,
            dtprofup: self.dtprofup.map(|i: String| {
                OfxParser::parse_custom_datetime(&i)
                    .inspect_err(|e: &String| error!("{e}"))
                    .unwrap_or_default()
            }),
            fi: self.fi.map(|i: FinancialInstitutionXml| i.into()),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
struct StatusXml {
    code: i32,
    severity: String,
    message: Option<String>,
}

impl Into<Status> for StatusXml {
    fn into(self) -> Status {
        Status {
            code: self.code,
            severity: self.severity,
            message: self.message,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
struct FinancialInstitutionXml {
    org: String,
    fid: Option<String>,
}

impl Into<FinancialInstitution> for FinancialInstitutionXml {
    fn into(self) -> FinancialInstitution {
        FinancialInstitution {
            org: self.org,
            fid: self.fid,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
struct StatementTransactionResponseXml {
    trnuid: String,
    status: StatusXml,
    stmtrs: StatementResponseXml,
}

impl Into<StatementTransactionResponse> for &StatementTransactionResponseXml {
    fn into(self) -> StatementTransactionResponse {
        StatementTransactionResponse {
            trnuid: self.trnuid.clone(),
            status: self.status.clone().into(),
            stmtrs: self.stmtrs.clone().into(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
struct StatementResponseXml {
    curdef: String,
    bankacctfrom: BankAccountFromXml,
    banktranlist: Option<BankTransactionListXml>,
    ledgerbal: Option<BalanceXml>,
    availbal: Option<BalanceXml>,
}

impl Into<StatementResponse> for StatementResponseXml {
    fn into(self) -> StatementResponse {
        StatementResponse {
            curdef: self.curdef,
            bankacctfrom: self.bankacctfrom.into(),
            banktranlist: self.banktranlist.map(|i: BankTransactionListXml| i.into()),
            ledgerbal: self.ledgerbal.map(|i: BalanceXml| i.into()),
            availbal: self.availbal.map(|i: BalanceXml| i.into()),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
struct BankAccountFromXml {
    bankid: String,
    acctid: String,
    accttype: String,
}

impl Into<BankAccount> for BankAccountFromXml {
    fn into(self) -> BankAccount {
        BankAccount {
            bankid: self.bankid,
            acctid: self.acctid,
            accttype: self.accttype,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
struct BankTransactionListXml {
    dtstart: String,
    dtend: String,
    stmttrn: Vec<TransactionXml>,
}

impl Into<BankTransactionList> for BankTransactionListXml {
    fn into(self) -> BankTransactionList {
        BankTransactionList {
            dtstart: OfxParser::parse_custom_datetime(&self.dtstart)
                .inspect_err(|e: &String| error!("{e}"))
                .unwrap_or_default(),
            dtend: OfxParser::parse_custom_datetime(&self.dtend)
                .inspect_err(|e: &String| error!("{e}"))
                .unwrap_or_default(),
            transactions: self
                .stmttrn
                .iter()
                .map(|t: &TransactionXml| t.into())
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
struct BalanceXml {
    balamt: f64,
    dtasof: String,
}

impl Into<Balance> for BalanceXml {
    fn into(self) -> Balance {
        Balance {
            balamt: self.balamt,
            dtasof: OfxParser::parse_custom_datetime(&self.dtasof)
                .inspect_err(|e: &String| error!("{}", e))
                .unwrap_or_default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
struct TransactionXml {
    trntype: String,
    dtposted: String,
    trnamt: f64,
    fitid: String,
    name: Option<String>,
    memo: Option<String>,
}

impl Into<Transaction> for &TransactionXml {
    fn into(self) -> Transaction {
        Transaction {
            trntype: self.trntype.clone(),
            dtposted: OfxParser::parse_custom_datetime(&self.dtposted)
                .inspect_err(|e| error!("Unable to parse dtserver date {}: {:?}", self.dtposted, e))
                .unwrap_or_default(),
            trnamt: self.trnamt,
            fitid: self.fitid.clone(),
            name: self.name.clone(),
            memo: self.memo.clone(),
        }
    }
}

pub struct OfxParser;

impl OfxParser {
    pub fn parse_string(content: &str) -> OfxResult<OfxDocument> {
        info!(
            "Parsing OFX content string. Length: {} bytes",
            content.len()
        );

        // Detect the start of the XML body using a regular expression
        let xml_start_regex = Regex::new(r"<\?xml.*\?>|<OFX>").unwrap();
        let xml_start_pos = xml_start_regex.find(content).map_or(0, |mat| mat.start());

        if xml_start_pos == 0 {
            error!(
                "No valid header found. The content starts with XML. Content: {}",
                content.trim()
            );
            return Err(OfxError::InvalidContent(
                "No valid header found".to_string(),
            ));
        }

        let (header_content, body_content) = content.split_at(xml_start_pos);

        let header = Self::parse_header(header_content)?;
        debug!("Successfully parsed header: {:#?}", header);

        // The body is XML content, starting from the second part
        let (signon, bank_msgs): (SignOnResponseXml, Vec<StatementTransactionResponseXml>) =
            Self::parse_xml_body(body_content)?;

        info!(
            "XML body parsed successfully. Signon messages: 1, Bank messages: {}",
            bank_msgs.len()
        );

        Ok(OfxDocument {
            header,
            signon: signon.into(),
            bank_msgs: bank_msgs
                .iter()
                .map(|i: &StatementTransactionResponseXml| i.into())
                .collect(),
        })
    }

    fn parse_header(header_content: &str) -> OfxResult<OfxHeader> {
        debug!("Parsing OFX header. Content: {}", header_content.trim());

        let mut version = None;
        let mut security = None;
        let mut encoding = None;
        let mut charset = None;
        let mut compression = None;
        let mut old_file_uid = None;
        let mut new_file_uid = None;

        for line in header_content.lines() {
            if line.trim().is_empty() {
                continue; // Skip empty lines for robustness
            }

            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() != 2 {
                warn!(
                    "Invalid header line format: {}. Skipping line.",
                    line.trim()
                );
                continue;
            }

            let key = parts[0].trim().to_uppercase(); // Normalize key to uppercase for case-insensitive matching
            let value = parts[1].trim();

            match key.as_str() {
                "OFXHEADER" => {
                    if value != "100" {
                        error!("Unsupported OFXHEADER value: {}. Expected '100'.", value);
                        return Err(OfxError::InvalidVersion(value.to_string()));
                    }
                    debug!("OFXHEADER validated: {}", value);
                }
                "VERSION" => {
                    if value != "102" {
                        error!("Unsupported OFX version: {}. Expected '102'.", value);
                        return Err(OfxError::InvalidVersion(value.to_string()));
                    }
                    version = Some(value.to_string());
                    debug!("VERSION set to: {}", value);
                }
                "SECURITY" => {
                    security = Some(value.to_string());
                    debug!("SECURITY set to: {}", value);
                }
                "ENCODING" => {
                    encoding = Some(value.to_string());
                    debug!("ENCODING set to: {}", value);
                }
                "CHARSET" => {
                    charset = Some(value.to_string());
                    debug!("CHARSET set to: {}", value);
                }
                "COMPRESSION" => {
                    compression = Some(value.to_string());
                    debug!("COMPRESSION set to: {}", value);
                }
                "OLDFILEUID" => {
                    old_file_uid = Some(value.to_string());
                    debug!("OLDFILEUID set to: {}", value);
                }
                "NEWFILEUID" => {
                    new_file_uid = Some(value.to_string());
                    debug!("NEWFILEUID set to: {}", value);
                }
                _ => {
                    warn!("Unknown header key: {}. Value: {}. Ignoring.", key, value);
                }
            }
        }

        // Validate required fields
        let version = match version {
            Some(v) => v,
            None => {
                error!("Missing required VERSION in header.");
                return Err(OfxError::MissingHeader("VERSION".to_string()));
            }
        };

        Ok(OfxHeader {
            version,
            security,
            encoding,
            charset,
            compression,
            old_file_uid,
            new_file_uid,
        })
    }

    fn parse_xml_body(
        body: &str,
    ) -> OfxResult<(SignOnResponseXml, Vec<StatementTransactionResponseXml>)> {
        info!(
            "Deserializing XML body. Sample: {:#?}",
            body.lines().collect::<Vec<_>>()
        );

        let ofx_xml: OfxXml = from_str(body)
            .inspect_err(|e| {
                error!(
                    "XML deserialization failed: {:?}. Body content may be malformed.",
                    e
                )
            })
            .map_err(|e| OfxError::Xml(e))?;

        debug!("XML deserialization successful. Extracting Signon and Bank messages.");

        Ok((ofx_xml.signonmsgsrsv1.sonrs, ofx_xml.bankmsgsrsv1.stmttrnrs))
    }

    pub fn parse_custom_datetime(s: &str) -> Result<DateTime<FixedOffset>, String> {
        // 1. Find the start of the timezone bracket '['
        if let Some(tz_start_index) = s.find('[') {
            // The datetime part: "20250604000000"
            let datetime_part = &s[..tz_start_index];

            // The inner timezone part: "-3:BRT"
            let tz_part = &s[tz_start_index + 1..s.len() - 1];

            // 2. Extract the numeric hour offset: "-3"
            if let Some(hour_offset_str) = tz_part.split(':').next() {
                let hour_offset: i32 = hour_offset_str
                    .parse()
                    .map_err(|e| format!("Unable to parse hour offset: {e:?}"))?;

                // 3. Format the offset into the `+hhmm` format required by `%z`
                // e.g., -3 becomes "-0300"
                let formatted_offset = format!("{hour_offset:+03}00");

                // 4. Create the final, parsable string: "20250604000000-0300"
                let parsable_string = format!("{datetime_part}{formatted_offset}");

                // 5. Parse the string using the correct format
                let format_str = "%Y%m%d%H%M%S%z";
                let date_time: DateTime<FixedOffset> =
                    DateTime::parse_from_str(&parsable_string, &format_str)
                        .map_err(|e| format!("Unble to parse processed date: {e:?}"))?;

                return Ok(date_time);
            }
        }

        Err("Invalid date format".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use rstest::rstest;
    use std::path::PathBuf;

    // #[rstest]
    // fn test_parse_valid_ofx_file() {
    //     let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    //     path.push("tests/fixtures/valid_ofx.ofx"); // Assume a test fixture file
    //
    //     let result = OfxParser::parse_file(path).unwrap();
    //
    //     assert_eq!(result.header.version, "102");
    //     assert!(result.signon.sonrs.status.code == 0); // Example assertion
    //     assert!(!result.bank_msgs.is_empty());
    // }
    //
    // #[rstest]
    // #[case("tests/fixtures/invalid_version.ofx", OfxError::InvalidVersion("103".to_string()))]
    // fn test_parse_invalid_ofx_file(#[case] path_str: &str, #[case] expected_error: OfxError) {
    //     let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    //     path.push(path_str);
    //
    //     let result = OfxParser::parse_file(path);
    //     assert!(result.is_err());
    //     assert_eq!(result.unwrap_err().to_string(), expected_error.to_string());
    // }
    //
    // #[rstest]
    // fn test_parse_string_with_invalid_header() {
    //     let invalid_content = "OFXHEADER:200\nINVALIDKEY:foo\n";
    //     let result = OfxParser::parse_string(invalid_content);
    //     assert!(matches!(result, Err(OfxError::InvalidVersion(_))));
    // }

    // Add more tests for edge cases, like missing VERSION or malformed XML
}
