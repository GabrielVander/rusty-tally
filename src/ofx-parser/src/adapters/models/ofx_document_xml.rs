use serde::Deserialize;

use crate::domain::entities::ofx::OfxHeader;

#[derive(Debug)]
pub struct OfxDocumentXml {
    pub header: OfxHeader,
    pub body: OfxBodyXml,
}

#[derive(Debug)]
pub struct OfxHeaderXml {
    pub version: String,
    pub security: Option<String>,
    pub encoding: Option<String>,
    pub charset: Option<String>,
    pub compression: Option<String>,
    pub old_file_uid: Option<String>,
    pub new_file_uid: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub struct OfxBodyXml {
    pub signonmsgsrsv1: SignOnMessageResponseV1Xml,
    pub bankmsgsrsv1: BankMessageResponseV1Xml,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub struct SignOnMessageResponseV1Xml {
    pub sonrs: SignOnResponseXml,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub struct BankMessageResponseV1Xml {
    pub stmttrnrs: Vec<StatementTransactionResponseXml>, // Matches the XML structure for multiple STMTTRNRS elements
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub struct SignOnResponseXml {
    pub status: StatusXml,
    pub dtserver: String,
    pub language: Option<String>,
    pub dtprofup: Option<String>,
    pub fi: Option<FinancialInstitutionXml>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub struct StatusXml {
    pub code: i32,
    pub severity: String,
    pub message: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub struct FinancialInstitutionXml {
    pub org: String,
    pub fid: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub struct StatementTransactionResponseXml {
    pub trnuid: String,
    pub status: StatusXml,
    pub stmtrs: StatementResponseXml,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub struct StatementResponseXml {
    pub curdef: String,
    pub bankacctfrom: BankAccountFromXml,
    pub banktranlist: Option<BankTransactionListXml>,
    pub ledgerbal: Option<BalanceXml>,
    pub availbal: Option<BalanceXml>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub struct BankAccountFromXml {
    pub bankid: String,
    pub acctid: String,
    pub accttype: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub struct BankTransactionListXml {
    pub dtstart: String,
    pub dtend: String,
    pub stmttrn: Vec<TransactionXml>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub struct BalanceXml {
    pub balamt: f64,
    pub dtasof: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub struct TransactionXml {
    pub trntype: String,
    pub dtposted: String,
    pub trnamt: f64,
    pub fitid: String,
    pub name: Option<String>,
    pub memo: Option<String>,
}
