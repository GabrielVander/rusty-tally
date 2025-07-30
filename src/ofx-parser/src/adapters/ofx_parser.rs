use crate::domain::entities::ofx::{
    Balance, BankAccount, BankTransactionList, FinancialInstitution, OfxBody, OfxDocument,
    OfxHeader, SignonResponse, StatementResponse, StatementTransactionResponse, Status,
    Transaction,
};
use chrono::{DateTime, FixedOffset};
use log::{debug, error, info, warn};
use quick_xml::de::from_str;
use regex::Regex;
use thiserror::Error;

use super::models::ofx_document_xml::{
    BalanceXml, BankAccountFromXml, BankTransactionListXml, FinancialInstitutionXml, OfxBodyXml,
    OfxDocumentXml, OfxHeaderXml, SignOnResponseXml, StatementResponseXml,
    StatementTransactionResponseXml, StatusXml, TransactionXml,
};

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

        let header: OfxHeader = Self::parse_header(header_content)?;
        debug!("Successfully parsed header: {header:#?}");

        // The body is XML content, starting from the second part
        let xml_body: OfxBodyXml = Self::parse_xml_body(body_content)?;

        let xml_document: OfxDocumentXml = OfxDocumentXml {
            header,
            body: xml_body,
        };

        Ok(OfxDocument::from(xml_document))
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
                        error!("Unsupported OFXHEADER value: {value}. Expected '100'.");
                        return Err(OfxError::InvalidVersion(value.to_string()));
                    }
                    debug!("OFXHEADER validated: {value}");
                }
                "VERSION" => {
                    if value != "102" {
                        error!("Unsupported OFX version: {value}. Expected '102'.");
                        return Err(OfxError::InvalidVersion(value.to_string()));
                    }
                    version = Some(value.to_string());
                    debug!("VERSION set to: {value}");
                }
                "SECURITY" => {
                    security = Some(value.to_string());
                    debug!("SECURITY set to: {value}");
                }
                "ENCODING" => {
                    encoding = Some(value.to_string());
                    debug!("ENCODING set to: {value}");
                }
                "CHARSET" => {
                    charset = Some(value.to_string());
                    debug!("CHARSET set to: {value}");
                }
                "COMPRESSION" => {
                    compression = Some(value.to_string());
                    debug!("COMPRESSION set to: {value}");
                }
                "OLDFILEUID" => {
                    old_file_uid = Some(value.to_string());
                    debug!("OLDFILEUID set to: {value}");
                }
                "NEWFILEUID" => {
                    new_file_uid = Some(value.to_string());
                    debug!("NEWFILEUID set to: {value}");
                }
                _ => {
                    warn!("Unknown header key: {key}. Value: {value}. Ignoring.");
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

    fn parse_xml_body(body: &str) -> OfxResult<OfxBodyXml> {
        info!(
            "Deserializing XML body. Sample: {:#?}",
            body.lines().collect::<Vec<_>>()
        );

        let ofx_xml: OfxBodyXml = from_str::<OfxBodyXml>(body)
            .inspect_err(|e| {
                error!("XML deserialization failed: {e:?}. Body content may be malformed.")
            })
            .map_err(OfxError::Xml)
            .inspect(|o| {
                debug!("Successfully deserialized XML body: {o:#?}");
            })?;

        Ok(ofx_xml)
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
                    DateTime::parse_from_str(&parsable_string, format_str)
                        .map_err(|e| format!("Unble to parse processed date: {e:?}"))?;

                return Ok(date_time);
            }
        }

        Err("Invalid date format".into())
    }
}

impl From<OfxDocumentXml> for OfxDocument {
    fn from(value: OfxDocumentXml) -> Self {
        OfxDocument {
            header: value.header.into(),
            body: value.body.into(),
        }
    }
}

impl From<OfxHeaderXml> for OfxHeader {
    fn from(value: OfxHeaderXml) -> Self {
        OfxHeader {
            version: value.version,
            security: value.security,
            encoding: value.encoding,
            charset: value.charset,
            compression: value.compression,
            old_file_uid: value.old_file_uid,
            new_file_uid: value.new_file_uid,
        }
    }
}

impl From<OfxBodyXml> for OfxBody {
    fn from(value: OfxBodyXml) -> Self {
        OfxBody {
            sign_on_response: value.signonmsgsrsv1.sonrs.into(),
            bank_msgs: value
                .bankmsgsrsv1
                .stmttrnrs
                .iter()
                .map(|i: &StatementTransactionResponseXml| i.into())
                .collect(),
        }
    }
}

impl From<SignOnResponseXml> for SignonResponse {
    fn from(value: SignOnResponseXml) -> Self {
        SignonResponse {
            status: value.status.into(),
            dtserver: OfxParser::parse_custom_datetime(&value.dtserver)
                .inspect_err(|e: &String| error!("{e}"))
                .unwrap_or_default(),
            language: value.language,
            dtprofup: value.dtprofup.map(|i: String| {
                OfxParser::parse_custom_datetime(&i)
                    .inspect_err(|e: &String| error!("{e}"))
                    .unwrap_or_default()
            }),
            fi: value.fi.map(|i: FinancialInstitutionXml| i.into()),
        }
    }
}

impl From<StatusXml> for Status {
    fn from(value: StatusXml) -> Self {
        Status {
            code: value.code,
            severity: value.severity,
            message: value.message,
        }
    }
}

impl From<FinancialInstitutionXml> for FinancialInstitution {
    fn from(value: FinancialInstitutionXml) -> Self {
        FinancialInstitution {
            org: value.org,
            fid: value.fid,
        }
    }
}

impl From<&StatementTransactionResponseXml> for StatementTransactionResponse {
    fn from(value: &StatementTransactionResponseXml) -> Self {
        StatementTransactionResponse {
            trnuid: value.trnuid.clone(),
            status: value.status.clone().into(),
            stmtrs: value.stmtrs.clone().into(),
        }
    }
}

impl From<StatementResponseXml> for StatementResponse {
    fn from(value: StatementResponseXml) -> Self {
        StatementResponse {
            curdef: value.curdef,
            bankacctfrom: value.bankacctfrom.into(),
            banktranlist: value.banktranlist.map(|i: BankTransactionListXml| i.into()),
            ledgerbal: value.ledgerbal.map(|i: BalanceXml| i.into()),
            availbal: value.availbal.map(|i: BalanceXml| i.into()),
        }
    }
}

impl From<BankAccountFromXml> for BankAccount {
    fn from(value: BankAccountFromXml) -> Self {
        BankAccount {
            bankid: value.bankid,
            acctid: value.acctid,
            accttype: value.accttype,
        }
    }
}

impl From<BankTransactionListXml> for BankTransactionList {
    fn from(value: BankTransactionListXml) -> Self {
        BankTransactionList {
            dtstart: OfxParser::parse_custom_datetime(&value.dtstart)
                .inspect_err(|e: &String| error!("{e}"))
                .unwrap_or_default(),
            dtend: OfxParser::parse_custom_datetime(&value.dtend)
                .inspect_err(|e: &String| error!("{e}"))
                .unwrap_or_default(),
            transactions: value
                .stmttrn
                .iter()
                .map(|t: &TransactionXml| t.into())
                .collect(),
        }
    }
}

impl From<BalanceXml> for Balance {
    fn from(value: BalanceXml) -> Self {
        Balance {
            balamt: value.balamt,
            dtasof: OfxParser::parse_custom_datetime(&value.dtasof)
                .inspect_err(|e: &String| error!("{e}"))
                .unwrap_or_default(),
        }
    }
}

impl From<&TransactionXml> for Transaction {
    fn from(value: &TransactionXml) -> Self {
        Transaction {
            trntype: value.trntype.clone(),
            dtposted: OfxParser::parse_custom_datetime(&value.dtposted)
                .inspect_err(|e| error!("Unable to parse dtserver date {}: {e:?}", value.dtposted))
                .unwrap_or_default(),
            trnamt: value.trnamt,
            fitid: value.fitid.clone(),
            name: value.name.clone(),
            memo: value.memo.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
