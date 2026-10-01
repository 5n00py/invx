#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Address {
    pub street: Option<String>,
    pub postal_code: Option<String>,
    pub city: Option<String>,
    pub country_code: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Party {
    pub name: String,
    pub address: Option<Address>,
    pub vat_id: Option<String>,
}
