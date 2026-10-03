# invx

`invx` is a Rust toolkit for reading, inspecting, validating, converting, and
comparing electronic invoices.

The project is built around a **canonical invoice model**. UBL 2.1 and
ebInterface 6.1 are parsed into format-specific representations and then mapped
into the same domain model.

```text
UBL 2.1 XML ─────────────┐
                         ▼
                  Canonical Invoice
                         ▲
ebInterface 6.1 XML ─────┘
```

Business logic works on the canonical model rather than directly on XML. This
keeps format syntax separate from invoice semantics and makes validation,
conversion, and comparison easier to reason about.

> `invx` is under active development. Format and validation support should be
> treated as a practical subset unless full conformance has been implemented
> and verified.

## Goals

`invx` is both a learning project and a practical e-invoicing toolkit.

The main goals are:

- understand e-invoice formats at the semantic level
- keep the domain model independent from XML syntax
- inspect invoices in a readable form
- validate canonical invoice semantics
- support useful EN 16931 business rules
- read and write UBL 2.1 and ebInterface 6.1
- convert between supported formats
- compare invoices semantically rather than as raw XML
- produce useful diagnostics for integration and consulting work

## Current status

Implemented today:

- UBL 2.1 parsing and writing
- ebInterface 6.1 parsing and writing
- automatic input format detection
- mapping both formats into the same canonical invoice model
- canonical JSON output
- human-readable inspection
- core validation
- EN 16931 subset validation
- semantic conversion
- conversion diagnostics
- semantic invoice comparison
- multiple invoice lines and VAT rates
- seller, buyer, addresses, VAT IDs, references, and payment information
- document-level allowances and charges
- line-level allowances and charges
- price base quantity
- canonical totals and VAT breakdowns
- semantic round-trip tests between supported formats

Still intentionally incomplete:

- broader UBL and ebInterface coverage
- broader EN 16931 coverage
- additional validation profiles
- XSD/schema validation
- official conformance validation
- additional invoice formats

## Architecture

The main rule is simple:

> XML formats do not define the domain model.

Each format has its own parser/writer model and mapping layer.

```text
UBL XML ───────▶ UBL model ───────┐
                                  │
                                  ▼
                           Canonical Invoice
                                  ▲
                                  │
ebInterface XML ─▶ ebInterface model
```

The canonical invoice is then used by the common functionality:

```text
Canonical Invoice
       │
       ├── inspect
       ├── validate
       ├── JSON
       ├── convert
       ├── write
       └── compare
```

This also means conversion does not happen directly from one XML tree to
another.

```text
source XML
    ↓
source parser
    ↓
canonical Invoice
    ↓
target diagnostics
    ↓
target writer
    ↓
target XML
```

## Canonical invoice model

The canonical model currently covers:

```text
Invoice
├── ID, issue date, currency
├── seller and buyer
├── order reference
├── invoice lines
│   ├── quantity and unit
│   ├── unit price
│   ├── price base quantity
│   ├── line allowances / charges
│   ├── line net amount
│   └── VAT information
├── document allowances / charges
├── VAT breakdown
├── payment information
└── totals
```

Money and quantities use `rust_decimal::Decimal`. Binary floating point is not
used for financial arithmetic.

Format-specific numeric and date values are parsed first and converted into
domain types in the mapping layer.

## Supported formats

### UBL 2.1

The supported UBL subset includes:

- invoice number and issue date
- document currency
- supplier and customer
- postal addresses and VAT IDs
- order reference
- invoice lines
- quantities and units
- unit prices and price base quantity
- line allowances and charges
- VAT categories and rates
- document allowances and charges
- VAT breakdowns
- payment means and payment account
- monetary totals

UBL output is written from the canonical model.

### ebInterface 6.1

The supported ebInterface subset includes:

- invoice number and date
- invoice currency
- biller and invoice recipient
- addresses and VAT IDs
- order reference
- item lists and invoice lines
- quantities and units
- `UnitPrice` with `BaseQuantity`
- line reductions and surcharges
- VAT information and breakdowns
- gross and payable totals
- universal bank transfer information

The ebInterface mapping preserves the order of line reductions and surcharges.

## Automatic format detection

CLI input does not need an explicit source format.

The input layer inspects the XML root element and namespace, selects the parser,
and maps the result into the canonical model.

```text
invoice.xml
    ↓
format detection
    ├── UBL 2.1
    └── ebInterface 6.1
    ↓
Canonical Invoice
```

## Installation

Once published on crates.io:

```bash
cargo install invx
```

To install the latest version directly from GitHub:

```bash
cargo install --git https://github.com/5n00py/invx.git
```

Then:

```bash
invx --help
```

## CLI

Inspect an invoice:

```bash
invx inspect invoice.xml
```

Validate with the core profile:

```bash
invx validate invoice.xml
```

Validate with the EN 16931 subset:

```bash
invx validate \
  --profile en16931-subset \
  invoice.xml
```

Render the canonical model as JSON:

```bash
invx to-json invoice.xml
```

Compare two invoices semantically:

```bash
invx compare invoice-a.xml invoice-b.xml
```

Conversion is also available through the CLI. Source format detection is
automatic; the target format is selected explicitly.

## Validation

### Core

The `core` profile checks canonical invoice consistency.

Current checks include:

- invoice contains at least one line
- line net totals reconcile with invoice totals
- document allowances and charges reconcile with invoice net
- net plus tax equals gross
- gross and payable totals are consistent with the current model
- currencies are consistent
- VAT breakdown totals reconcile with total tax
- price base quantity is positive
- line arithmetic is consistent

Line arithmetic follows the canonical model:

```text
quantity × unit price / price base quantity
- line allowances
+ line charges
= line net amount
```

An omitted price base quantity means `1`.

### EN 16931 subset

The `en16931-subset` profile applies core validation plus selected EN 16931 and
aligned business rules.

Current checks include:

- payment account requirements for credit transfer
- invoice-line VAT category
- standard-rated VAT rate
- VAT breakdown presence and VAT amount calculation
- non-negative item net price
- price base quantity unit consistency
- allowance/charge base amount and percentage consistency
- allowance/charge amount calculation

This is deliberately a subset. `invx` does not currently claim complete
EN 16931 conformance.

## Allowances, charges, and price base quantity

Allowances and charges are represented explicitly rather than folded into the
price.

A line may contain:

```text
quantity
× unit price
÷ price base quantity
- allowances
+ charges
= line net amount
```

Both line and document adjustments can carry:

- amount
- optional base amount
- optional percentage
- reason code
- one or more reasons

Document adjustments can additionally carry their own tax information.

## Conversion diagnostics

Before writing a target format, `invx` checks whether the canonical invoice can
be represented safely.

Diagnostics are classified as:

```text
Blocking       target cannot represent the invoice safely
Lossy          conversion is possible but information is reduced
Informational  representation changes without changing business meaning
```

Examples include:

- unsupported ebInterface line IDs or units
- missing adjustment base amounts
- incompatible price base quantity units
- numeric precision that would require rounding
- currency mismatches
- multiple reasons that must be combined into one target field
- syntax-specific payment information that changes representation

Blocking diagnostics stop conversion before the writer runs.

## Semantic comparison

Comparison happens on canonical invoice semantics rather than raw XML.

That means two invoices can compare equal even when their syntax differs. For
example, an omitted price base quantity and an explicit base quantity of `1`
are treated as equivalent where they mean the same thing.

Comparison currently covers parties, lines, pricing, line adjustments, VAT,
document adjustments, payment semantics, and totals.

Syntax-specific details that do not change business meaning can be ignored
deliberately, such as a UBL payment means code when both invoices still
represent the same bank-transfer semantics.

## Error handling

`invx` prefers explicit errors over silent information loss.

If a source concept cannot be represented safely in the canonical model, or the
target format cannot represent a canonical concept, the operation should fail
or return a clear diagnostic instead of dropping data.

CLI exit codes follow this convention:

```text
0  success / valid / equal
1  processed but invalid / different / incompatible
2  processing or application error
```

## Development

Run the full checks with:

```bash
cargo fmt
cargo clippy --all-targets --all-features
cargo test
```

Inspect the test fixtures manually with:

```bash
cargo run -- inspect tests/fixtures/ubl/simple-invoice.xml
```

or:

```bash
cargo run -- inspect \
  tests/fixtures/ebinterface/simple-invoice.xml
```

Both go through the same canonical invoice pipeline.

## Design principles

### Canonical semantics first

Format models exist to read and write their syntax. Business logic operates on
the canonical invoice.

### Preserve information

Do not silently discard meaningful source data. Reject or diagnose mappings that
cannot be represented safely.

### Exact financial arithmetic

Use decimal arithmetic for financial values.

### Small vertical slices

Features are implemented end to end:

```text
format model
    ↓
mapping
    ↓
canonical model
    ↓
validation / conversion / comparison
    ↓
tests
```

### Do not overstate compliance

Support for a standard should only be described as complete when the relevant
syntax and semantic requirements have actually been implemented and verified.

## Standards

The project currently works with:

- OASIS UBL 2.1
- ebInterface 6.1
- EN 16931 concepts and selected business rules

`invx` is an independent project and is not an official implementation or
certification tool for these standards.

## License

Licensed under either of:

- Apache License, Version 2.0
- MIT License

at your option.

Copyright © 2026 LNL join.tech FlexCo.
