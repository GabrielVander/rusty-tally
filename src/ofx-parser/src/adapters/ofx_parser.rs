use crate::domain::entities::ofx::{
    Balance, BankAccount, BankTransactionList, FinancialInstitution, OfxDocument, OfxHeader,
    SignonResponse, StatementResponse, StatementTransactionResponse, Status, Transaction,
};
use chrono::{DateTime, FixedOffset};
use log::{debug, error, info, warn};
use quick_xml::de::from_str;
use regex::Regex;
use serde::Deserialize;
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
                .inspect_err(|e: &String| error!("{e}"))
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
        debug!("Successfully parsed header: {header:#?}");

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
                error!("XML deserialization failed: {e:?}. Body content may be malformed.")
            })
            .map_err(OfxError::Xml)?;

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
}
