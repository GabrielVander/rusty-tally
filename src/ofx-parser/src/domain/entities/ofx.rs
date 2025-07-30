use chrono::{DateTime, FixedOffset};

#[derive(Debug, Clone, PartialEq)]
pub struct OfxHeader {
    pub version: String,
    pub security: Option<String>,
    pub encoding: Option<String>,
    pub charset: Option<String>,
    pub compression: Option<String>,
    pub old_file_uid: Option<String>,
    pub new_file_uid: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SignonMessage {
    pub sonrs: SignonResponse,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SignonResponse {
    pub status: Status,
    pub dtserver: DateTime<FixedOffset>,
    pub language: Option<String>,
    pub dtprofup: Option<DateTime<FixedOffset>>,
    pub fi: Option<FinancialInstitution>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Status {
    pub code: i32,
    pub severity: String,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FinancialInstitution {
    pub org: String,
    pub fid: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BankAccount {
    pub bankid: String,
    pub acctid: String,
    pub accttype: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Transaction {
    pub trntype: String,
    pub dtposted: DateTime<FixedOffset>,
    pub trnamt: f64,
    pub fitid: String,
    pub name: Option<String>,
    pub memo: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BankTransactionList {
    pub dtstart: DateTime<FixedOffset>,
    pub dtend: DateTime<FixedOffset>,
    pub transactions: Vec<Transaction>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StatementTransactionResponse {
    pub trnuid: String,
    pub status: Status,
    pub stmtrs: StatementResponse,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StatementResponse {
    pub curdef: String,
    pub bankacctfrom: BankAccount,
    pub banktranlist: Option<BankTransactionList>,
    pub ledgerbal: Option<Balance>,
    pub availbal: Option<Balance>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Balance {
    pub balamt: f64,
    pub dtasof: DateTime<FixedOffset>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OfxDocument {
    pub header: OfxHeader,
    pub signon: SignonResponse,
    pub bank_msgs: Vec<StatementTransactionResponse>,
}
