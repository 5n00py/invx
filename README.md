# invx

`invx` is a Rust toolkit for reading, inspecting, validating, transforming, and
comparing electronic invoices.

The project is built around a **canonical invoice model**. Format-specific XML
such as UBL 2.1 or ebInterface 6.1 is parsed into its own representation and
then mapped into the same canonical domain model.

```text
UBL 2.1 XML ─────────────┐
                         │
                         ▼
                  Canonical Invoice
                         ▲
                         │
ebInterface 6.1 XML ─────┘
```

This keeps invoice semantics separate from the syntax used to represent them
and provides a foundation for validation, conversion, comparison, and
consulting/debugging workflows.

> `invx` is currently under active development. Supported standards and
> validation profiles are intentionally described as subsets unless full
> conformance has been implemented and verified.

## Goals

`invx` is intended as both a learning project and a practical e-invoicing
toolkit.

The main goals are:

- understand e-invoice formats at the semantic level
- provide a format-independent canonical invoice model
- inspect invoices in a human-readable way
- validate canonical invoice semantics
- support EN 16931 business rules
- read and write UBL 2.1 invoices
- read and write ebInterface 6.1 invoices
- convert between supported invoice formats
- compare invoices semantically rather than as raw XML
- provide useful diagnostics for integration and consulting work

## Current status

Currently implemented:

- UBL 2.1 invoice parsing
- ebInterface 6.1 invoice parsing
- automatic invoice format detection
- mapping both formats into the same canonical invoice model
- multiple invoice lines
- multiple VAT rates
- seller and buyer information
- postal addresses
- VAT identifiers
- order references
- payment information
- document-level allowances and charges for UBL
- canonical invoice totals
- canonical validation
- EN 16931 subset validation
- human-readable invoice inspection
- canonical JSON output

Planned next:

- canonical model → UBL 2.1 XML writer
- canonical model → ebInterface 6.1 XML writer
- broader UBL 2.1 invoice coverage
- broader ebInterface 6.1 coverage
- broader EN 16931 semantic coverage
- semantic invoice conversion
- semantic invoice comparison
- additional validation profiles
- schema and official conformance validation

## Architecture

The main architectural rule is:

> XML formats do not define the domain model.

Each supported format has its own parser representation and mapping layer.

```text
                        ┌─────────────────────┐
UBL XML ──────────────▶│ UBL parser model    │
                        └──────────┬──────────┘
                                  │
                                  ▼
                         ┌─────────────────┐
                         │                 │
                         │ Canonical       │
                         │ Invoice         │
                         │                 │
                         └─────────────────┘
                                  ▲
                                  │
                        ┌──────────┴──────────┐
ebInterface XML ──────▶│ ebInterface model   │
                        └─────────────────────┘
```

The canonical model is then consumed by common functionality:

```text
Canonical Invoice
       │
       ├── inspect
       ├── validate
       ├── JSON
       ├── writers
       ├── conversion
       └── semantic comparison
```

This prevents format-specific concepts from leaking unnecessarily into the rest
of the application.

## Canonical invoice model

The canonical domain model currently represents concepts such as:

```text
Invoice
├── invoice ID
├── issue date
├── currency
├── seller
├── buyer
├── order reference
├── invoice lines
├── document adjustments
├── VAT breakdown
├── payment information
└── totals
```

Money is represented using `rust_decimal::Decimal`.

Floating-point values are deliberately avoided for financial amounts.

Format-specific numeric and date values are initially parsed as strings and
converted into domain types in the mapping layer.

## Supported input formats

### UBL 2.1

`invx` currently supports a useful subset of UBL 2.1 Invoice documents.

Examples of currently mapped concepts include:

- invoice number
- issue date
- document currency
- supplier
- customer
- addresses
- VAT identifiers
- order reference
- invoice lines
- quantities and units
- prices
- VAT categories and rates
- VAT breakdowns
- payment means
- payment account
- document allowances and charges
- monetary totals

Support is being expanded toward comprehensive UBL 2.1 e-invoice coverage.

### ebInterface 6.1

`invx` can also read ebInterface 6.1 invoices and map them into the same
canonical invoice model.

Currently mapped concepts include:

- invoice number
- issue date
- invoice currency
- biller
- invoice recipient
- addresses
- VAT identifiers
- order reference
- item lists
- invoice lines
- quantities and units
- prices
- VAT information
- VAT breakdowns
- gross and payable totals
- universal bank transfer information

Support is being expanded toward comprehensive ebInterface 6.1 coverage.

## Automatic format detection

CLI commands do not need to know the source invoice format.

The shared input layer inspects the XML root element and namespace and
dispatches to the corresponding parser automatically.

Conceptually:

```text
invoice.xml
    │
    ▼
format detection
    │
    ├── UBL 2.1
    │      ↓
    │   UBL parser
    │
    └── ebInterface 6.1
           ↓
        ebInterface parser

              ↓

       Canonical Invoice
```

As a result, the same CLI commands work for all supported input formats.

## CLI

### Inspect an invoice

```bash
cargo run -- inspect invoice.xml
```

or, once installed:

```bash
invx inspect invoice.xml
```

Example output:

```text
Document
  Invoice ID: 2026-00421
  Issue date: 2026-09-30
  Currency:   EUR
  Order ref:  PO-4711

Seller
  Example Supplier GmbH
  Supplier Street 1
  1010 Vienna
  AT
  VAT ID: ATU12345678

Buyer
  Example Logistics GmbH
  Customer Street 10
  1020 Vienna
  AT
  VAT ID: ATU87654321

Lines
  ...

VAT
  ...

Payment
  ...

Totals
  ...
```

### Validate an invoice

Core validation:

```bash
invx validate invoice.xml
```

EN 16931 subset:

```bash
invx validate \
  --profile en16931-subset \
  invoice.xml
```

Successful validation produces output such as:

```text
VALID [core]: invoice.xml
```

Validation failure produces violations such as:

```text
INVALID [core]: invoice.xml
ERROR CORE-002 [totals.gross_amount] ...
```

## Validation profiles

### Core

The `core` profile validates internal canonical invoice consistency.

Examples include:

- invoices contain lines
- line totals agree with invoice totals
- allowances and charges reconcile with the net total
- net amount plus tax equals gross amount
- gross and payable totals are consistent with the currently supported model
- currencies are consistent
- VAT breakdown totals reconcile with the invoice tax total

### EN 16931 subset

The `en16931-subset` profile applies core validation plus currently implemented
EN 16931-inspired business rules.

Current rules include checks around:

- bank-transfer payment accounts
- invoice-line VAT categories
- standard-rated VAT rates
- VAT breakdown presence
- VAT amount calculations

This profile is deliberately called a **subset**.

`invx` does not currently claim complete EN 16931 conformance.

## Canonical JSON

Any supported invoice format can be converted into the canonical JSON
representation:

```bash
invx to-json invoice.xml
```

For example:

```json
{
  "id": "2026-00421",
  "issue_date": "2026-09-30",
  "currency": "EUR",
  "seller": {
    "name": "Example Supplier GmbH",
    "vat_id": "ATU12345678"
  },
  "buyer": {
    "name": "Example Logistics GmbH",
    "vat_id": "ATU87654321"
  },
  "lines": [],
  "adjustments": [],
  "vat_breakdown": [],
  "totals": {}
}
```

The JSON represents the **canonical invoice**, not the source XML structure.

This makes it useful for:

- debugging mappings
- understanding invoices
- APIs
- tests
- semantic comparisons
- future conversion workflows

## Payment model

The canonical payment model distinguishes between semantic payment methods and
syntax-specific codes.

For example:

```text
UBL PaymentMeansCode = 58
        ↓
PaymentMethod::BankTransfer
means_code = Some("58")
```

while ebInterface may express the same concept structurally:

```text
UniversalBankTransaction
        ↓
PaymentMethod::BankTransfer
means_code = None
```

This allows syntax-specific information to be retained without making the
canonical model depend on UBL.

## Document adjustments

The canonical model distinguishes between:

```text
Allowance
Charge
```

and represents:

- amount
- optional base amount
- optional percentage
- reason
- reason code
- tax information

Canonical totals explicitly distinguish:

```text
line net
allowances
charges
net
tax
gross
payable
```

with the relationship:

```text
line net
- allowances
+ charges
= net

net
+ tax
= gross
```

## Conversion

Semantic conversion is a major goal of `invx`.

The planned architecture is:

```text
source XML
    ↓
source-specific parser
    ↓
canonical Invoice
    ↓
target-specific writer
    ↓
target XML
```

For example:

```text
UBL 2.1
   ↓
canonical Invoice
   ↓
ebInterface 6.1
```

Writers will target the canonical model rather than converting directly between
XML trees.

This makes unsupported or lossy mappings explicit and allows the same writer to
work regardless of the original source format.

## Semantic comparison

A future `compare` command will compare invoices based on canonical semantics
rather than XML syntax.

Planned usage:

```bash
invx compare invoice-ubl.xml invoice-ebinterface.xml
```

Two documents may differ significantly as XML while still representing the same
business invoice.

## Error handling

`invx` distinguishes between:

- processing/application errors
- successfully processed but invalid invoices
- successful processing and validation

CLI exit codes follow this general convention:

```text
0  success / valid
1  invoice processed but invalid
2  processing or application error
```

Mapping code prefers explicit errors over silent data loss.

For example, if a source format contains multiple values but the canonical
model currently supports only one, the mapper should reject the document rather
than silently discard information.

## Development

Run the full development checks with:

```bash
cargo fmt
cargo clippy --all-targets --all-features
cargo test
```

Run a specific invoice manually with:

```bash
cargo run -- inspect tests/fixtures/ubl/simple-invoice.xml
```

or:

```bash
cargo run -- inspect \
  tests/fixtures/ebinterface/simple-invoice.xml
```

Both should travel through the same canonical invoice pipeline.

## Design principles

### Canonical semantics first

Format-specific XML models exist to parse and write their corresponding syntax.

Business logic operates on the canonical invoice.

### Preserve information

Mapping should not silently discard meaningful source data.

If a mapping cannot currently be represented safely, returning an explicit
error is preferable.

### Exact financial arithmetic

Financial values use decimal arithmetic rather than binary floating point.

### Small vertical slices

Features are implemented end-to-end:

```text
XML
 ↓
format model
 ↓
mapping
 ↓
canonical model
 ↓
validation / CLI / tests
```

### Do not overstate compliance

Support for a format or standard should only be described as complete once the
relevant syntax and semantic requirements have been implemented and verified.

## Standards

The project is designed around standards including:

- OASIS UBL 2.1
- ebInterface 6.1
- EN 16931

`invx` is an independent project and is not an official implementation or
certification tool for these standards.
