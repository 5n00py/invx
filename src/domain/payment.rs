use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PaymentInformation {
    /// Payment means type code, e.g. UNCL4461 code "58" for SEPA credit transfer.
    pub means_code: String,

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
