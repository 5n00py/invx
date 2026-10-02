use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentMethod {
    BankTransfer,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PaymentInformation {
    pub method: PaymentMethod,

    /// Original standardized payment means code when one exists,
    /// e.g. UBL / EN 16931 "30" or "58".
    pub means_code: Option<String>,

    /// Remittance information used to associate the payment with the invoice.
    pub reference: Option<String>,

    /// Account to which the payment should be made.
    pub payee_account: Option<PaymentAccount>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PaymentAccount {
    /// Payment account identifier, such as an IBAN or BBAN.
    pub identifier: String,
}
