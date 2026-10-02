mod result;

use std::collections::{BTreeMap, BTreeSet};

use rust_decimal::Decimal;

use crate::domain::{
    AdjustmentKind, DocumentAdjustment, Invoice, InvoiceLine, Money, Party, PaymentInformation,
    PaymentMethod, TaxInformation, VatBreakdown,
};

pub use result::{ComparisonResult, Difference};

pub fn compare(left: &Invoice, right: &Invoice) -> ComparisonResult {
    let mut result = ComparisonResult::default();

    compare_rendered(
        &mut result,
        "id",
        Some(left.id.as_str().to_string()),
        Some(right.id.as_str().to_string()),
    );

    compare_rendered(
        &mut result,
        "issue_date",
        Some(left.issue_date.to_string()),
        Some(right.issue_date.to_string()),
    );

    compare_rendered(
        &mut result,
        "currency",
        Some(left.currency.as_str().to_string()),
        Some(right.currency.as_str().to_string()),
    );

    compare_party(&mut result, "seller", &left.seller, &right.seller);

    compare_party(&mut result, "buyer", &left.buyer, &right.buyer);

    compare_rendered(
        &mut result,
        "order_reference",
        left.order_reference.clone(),
        right.order_reference.clone(),
    );

    compare_lines(&mut result, &left.lines, &right.lines);

    compare_adjustments(&mut result, &left.adjustments, &right.adjustments);

    compare_vat_breakdown(&mut result, &left.vat_breakdown, &right.vat_breakdown);

    compare_payment(&mut result, left.payment.as_ref(), right.payment.as_ref());

    compare_money(
        &mut result,
        "totals.line_net_amount",
        &left.totals.line_net_amount,
        &right.totals.line_net_amount,
    );

    compare_money(
        &mut result,
        "totals.allowance_amount",
        &left.totals.allowance_amount,
        &right.totals.allowance_amount,
    );

    compare_money(
        &mut result,
        "totals.charge_amount",
        &left.totals.charge_amount,
        &right.totals.charge_amount,
    );

    compare_money(
        &mut result,
        "totals.net_amount",
        &left.totals.net_amount,
        &right.totals.net_amount,
    );

    compare_money(
        &mut result,
        "totals.tax_amount",
        &left.totals.tax_amount,
        &right.totals.tax_amount,
    );

    compare_money(
        &mut result,
        "totals.gross_amount",
        &left.totals.gross_amount,
        &right.totals.gross_amount,
    );

    compare_money(
        &mut result,
        "totals.payable_amount",
        &left.totals.payable_amount,
        &right.totals.payable_amount,
    );

    result
}

fn compare_party(result: &mut ComparisonResult, path: &str, left: &Party, right: &Party) {
    compare_rendered(
        result,
        &format!("{path}.name"),
        Some(left.name.clone()),
        Some(right.name.clone()),
    );

    compare_rendered(
        result,
        &format!("{path}.vat_id"),
        left.vat_id.clone(),
        right.vat_id.clone(),
    );

    match (left.address.as_ref(), right.address.as_ref()) {
        (Some(left_address), Some(right_address)) => {
            compare_rendered(
                result,
                &format!("{path}.address.street"),
                left_address.street.clone(),
                right_address.street.clone(),
            );

            compare_rendered(
                result,
                &format!("{path}.address.postal_code"),
                left_address.postal_code.clone(),
                right_address.postal_code.clone(),
            );

            compare_rendered(
                result,
                &format!("{path}.address.city"),
                left_address.city.clone(),
                right_address.city.clone(),
            );

            compare_rendered(
                result,
                &format!("{path}.address.country_code"),
                left_address.country_code.clone(),
                right_address.country_code.clone(),
            );
        }

        (Some(_), None) => {
            result.push(Difference::new(
                format!("{path}.address"),
                Some("present".to_string()),
                None,
            ));
        }

        (None, Some(_)) => {
            result.push(Difference::new(
                format!("{path}.address"),
                None,
                Some("present".to_string()),
            ));
        }

        (None, None) => {}
    }
}

fn compare_lines(result: &mut ComparisonResult, left: &[InvoiceLine], right: &[InvoiceLine]) {
    let left_groups = group_lines(left);

    let right_groups = group_lines(right);

    let ids = left_groups
        .keys()
        .chain(right_groups.keys())
        .map(|id| (*id).to_string())
        .collect::<BTreeSet<_>>();

    for id in ids {
        match (left_groups.get(id.as_str()), right_groups.get(id.as_str())) {
            (Some(left_lines), Some(right_lines)) => {
                if left_lines.len() != right_lines.len() {
                    compare_rendered(
                        result,
                        &format!("lines[id={id}].count"),
                        Some(left_lines.len().to_string()),
                        Some(right_lines.len().to_string()),
                    );
                }

                let unique = left_lines.len() == 1 && right_lines.len() == 1;

                for (index, (left_line, right_line)) in
                    left_lines.iter().zip(right_lines.iter()).enumerate()
                {
                    let path = if unique {
                        format!("lines[id={id}]")
                    } else {
                        format!("lines[id={id}][{index}]")
                    };

                    compare_line(result, &path, left_line, right_line);
                }
            }

            (Some(left_lines), None) => {
                for (index, line) in left_lines.iter().enumerate() {
                    let path = if left_lines.len() == 1 {
                        format!("lines[id={id}]")
                    } else {
                        format!("lines[id={id}][{index}]")
                    };

                    result.push(Difference::new(path, Some(line_summary(line)), None));
                }
            }

            (None, Some(right_lines)) => {
                for (index, line) in right_lines.iter().enumerate() {
                    let path = if right_lines.len() == 1 {
                        format!("lines[id={id}]")
                    } else {
                        format!("lines[id={id}][{index}]")
                    };

                    result.push(Difference::new(path, None, Some(line_summary(line))));
                }
            }

            (None, None) => {}
        }
    }
}

fn group_lines(lines: &[InvoiceLine]) -> BTreeMap<&str, Vec<&InvoiceLine>> {
    let mut grouped = BTreeMap::new();

    for line in lines {
        grouped
            .entry(line.id.as_str())
            .or_insert_with(Vec::new)
            .push(line);
    }

    grouped
}

fn compare_line(
    result: &mut ComparisonResult,
    path: &str,
    left: &InvoiceLine,
    right: &InvoiceLine,
) {
    compare_rendered(
        result,
        &format!("{path}.description"),
        Some(left.description.clone()),
        Some(right.description.clone()),
    );

    compare_decimal(
        result,
        &format!("{path}.quantity"),
        left.quantity,
        right.quantity,
    );

    compare_rendered(
        result,
        &format!("{path}.unit_code"),
        left.unit_code.clone(),
        right.unit_code.clone(),
    );

    compare_money(
        result,
        &format!("{path}.unit_price"),
        &left.unit_price,
        &right.unit_price,
    );

    compare_money(
        result,
        &format!("{path}.net_amount"),
        &left.net_amount,
        &right.net_amount,
    );

    compare_tax_information(result, &format!("{path}.tax"), &left.tax, &right.tax);
}

fn compare_vat_breakdown(
    result: &mut ComparisonResult,
    left: &[VatBreakdown],
    right: &[VatBreakdown],
) {
    let left_groups = group_vat(left);

    let right_groups = group_vat(right);

    let keys = left_groups
        .keys()
        .chain(right_groups.keys())
        .cloned()
        .collect::<BTreeSet<_>>();

    for key in keys {
        match (left_groups.get(&key), right_groups.get(&key)) {
            (Some(left_items), Some(right_items)) => {
                if left_items.len() != right_items.len() {
                    compare_rendered(
                        result,
                        &format!("vat_breakdown[{key}].count"),
                        Some(left_items.len().to_string()),
                        Some(right_items.len().to_string()),
                    );
                }

                let unique = left_items.len() == 1 && right_items.len() == 1;

                for (index, (left_vat, right_vat)) in
                    left_items.iter().zip(right_items.iter()).enumerate()
                {
                    let path = if unique {
                        format!("vat_breakdown[{key}]")
                    } else {
                        format!("vat_breakdown[{key}][{index}]")
                    };

                    compare_money(
                        result,
                        &format!("{path}.taxable_amount"),
                        &left_vat.taxable_amount,
                        &right_vat.taxable_amount,
                    );

                    compare_money(
                        result,
                        &format!("{path}.tax_amount"),
                        &left_vat.tax_amount,
                        &right_vat.tax_amount,
                    );
                }
            }

            (Some(left_items), None) => {
                for (index, vat) in left_items.iter().enumerate() {
                    let path = if left_items.len() == 1 {
                        format!("vat_breakdown[{key}]")
                    } else {
                        format!("vat_breakdown[{key}][{index}]")
                    };

                    result.push(Difference::new(path, Some(vat_summary(vat)), None));
                }
            }

            (None, Some(right_items)) => {
                for (index, vat) in right_items.iter().enumerate() {
                    let path = if right_items.len() == 1 {
                        format!("vat_breakdown[{key}]")
                    } else {
                        format!("vat_breakdown[{key}][{index}]")
                    };

                    result.push(Difference::new(path, None, Some(vat_summary(vat))));
                }
            }

            (None, None) => {}
        }
    }
}

fn group_vat(items: &[VatBreakdown]) -> BTreeMap<String, Vec<&VatBreakdown>> {
    let mut grouped = BTreeMap::new();

    for item in items {
        let category = item.category_code.as_deref().unwrap_or("<none>");

        let key = format!("category={category},rate={}", decimal_string(item.rate,));

        grouped.entry(key).or_insert_with(Vec::new).push(item);
    }

    grouped
}

fn compare_adjustments(
    result: &mut ComparisonResult,
    left: &[DocumentAdjustment],
    right: &[DocumentAdjustment],
) {
    if left.len() != right.len() {
        compare_rendered(
            result,
            "adjustments.count",
            Some(left.len().to_string()),
            Some(right.len().to_string()),
        );
    }

    for (index, (left_adjustment, right_adjustment)) in left.iter().zip(right.iter()).enumerate() {
        let path = format!("adjustments[{index}]");

        compare_rendered(
            result,
            &format!("{path}.kind"),
            Some(adjustment_kind(left_adjustment.kind).to_string()),
            Some(adjustment_kind(right_adjustment.kind).to_string()),
        );

        compare_money(
            result,
            &format!("{path}.amount"),
            &left_adjustment.amount,
            &right_adjustment.amount,
        );

        compare_optional_money(
            result,
            &format!("{path}.base_amount"),
            left_adjustment.base_amount.as_ref(),
            right_adjustment.base_amount.as_ref(),
        );

        compare_optional_decimal(
            result,
            &format!("{path}.percentage"),
            left_adjustment.percentage,
            right_adjustment.percentage,
        );

        compare_rendered(
            result,
            &format!("{path}.reason_code"),
            left_adjustment.reason_code.clone(),
            right_adjustment.reason_code.clone(),
        );

        compare_rendered(
            result,
            &format!("{path}.reasons"),
            Some(left_adjustment.reasons.join(" | ")),
            Some(right_adjustment.reasons.join(" | ")),
        );

        compare_optional_tax(
            result,
            &format!("{path}.tax"),
            left_adjustment.tax.as_ref(),
            right_adjustment.tax.as_ref(),
        );
    }

    /*
     * If one side has more adjustments than the
     * other, the count difference above tells us
     * that already. We also report the unmatched
     * adjustment itself for a more useful diff.
     */

    if left.len() > right.len() {
        for (index, adjustment) in left.iter().enumerate().skip(right.len()) {
            result.push(Difference::new(
                format!("adjustments[{index}]"),
                Some(adjustment_summary(adjustment)),
                None,
            ));
        }
    }

    if right.len() > left.len() {
        for (index, adjustment) in right.iter().enumerate().skip(left.len()) {
            result.push(Difference::new(
                format!("adjustments[{index}]"),
                None,
                Some(adjustment_summary(adjustment)),
            ));
        }
    }
}

fn compare_payment(
    result: &mut ComparisonResult,
    left: Option<&PaymentInformation>,
    right: Option<&PaymentInformation>,
) {
    match (left, right) {
        (Some(left_payment), Some(right_payment)) => {
            compare_rendered(
                result,
                "payment.method",
                Some(payment_method(left_payment.method).to_string()),
                Some(payment_method(right_payment.method).to_string()),
            );

            /*
             * means_code is intentionally not
             * compared.
             *
             * It is syntax-specific metadata:
             *
             * UBL:
             *   BankTransfer + Some("30"/"58")
             *
             * ebInterface:
             *   BankTransfer + None
             *
             * Those can still represent the same
             * business payment semantics.
             */

            compare_rendered(
                result,
                "payment.reference",
                left_payment.reference.clone(),
                right_payment.reference.clone(),
            );

            compare_rendered(
                result,
                "payment.payee_account",
                left_payment
                    .payee_account
                    .as_ref()
                    .map(|account| account.identifier.clone()),
                right_payment
                    .payee_account
                    .as_ref()
                    .map(|account| account.identifier.clone()),
            );
        }

        (Some(_), None) => {
            result.push(Difference::new(
                "payment",
                Some("present".to_string()),
                None,
            ));
        }

        (None, Some(_)) => {
            result.push(Difference::new(
                "payment",
                None,
                Some("present".to_string()),
            ));
        }

        (None, None) => {}
    }
}

fn compare_tax_information(
    result: &mut ComparisonResult,
    path: &str,
    left: &TaxInformation,
    right: &TaxInformation,
) {
    compare_rendered(
        result,
        &format!("{path}.category_code"),
        left.category_code.clone(),
        right.category_code.clone(),
    );

    compare_decimal(result, &format!("{path}.rate"), left.rate, right.rate);
}

fn compare_optional_tax(
    result: &mut ComparisonResult,
    path: &str,
    left: Option<&TaxInformation>,
    right: Option<&TaxInformation>,
) {
    match (left, right) {
        (Some(left_tax), Some(right_tax)) => {
            compare_tax_information(result, path, left_tax, right_tax);
        }

        (Some(_), None) => {
            result.push(Difference::new(path, Some("present".to_string()), None));
        }

        (None, Some(_)) => {
            result.push(Difference::new(path, None, Some("present".to_string())));
        }

        (None, None) => {}
    }
}

fn compare_money(result: &mut ComparisonResult, path: &str, left: &Money, right: &Money) {
    /*
     * Compare the actual domain values rather
     * than their textual rendering.
     *
     * Decimal considers 20, 20.0 and 20.00
     * numerically equal.
     */
    if left.amount != right.amount || left.currency != right.currency {
        result.push(Difference::new(
            path,
            Some(money_string(left)),
            Some(money_string(right)),
        ));
    }
}

fn compare_optional_money(
    result: &mut ComparisonResult,
    path: &str,
    left: Option<&Money>,
    right: Option<&Money>,
) {
    match (left, right) {
        (Some(left_money), Some(right_money)) => {
            compare_money(result, path, left_money, right_money);
        }

        (Some(left_money), None) => {
            result.push(Difference::new(path, Some(money_string(left_money)), None));
        }

        (None, Some(right_money)) => {
            result.push(Difference::new(path, None, Some(money_string(right_money))));
        }

        (None, None) => {}
    }
}

fn compare_decimal(result: &mut ComparisonResult, path: &str, left: Decimal, right: Decimal) {
    if left != right {
        result.push(Difference::new(
            path,
            Some(decimal_string(left)),
            Some(decimal_string(right)),
        ));
    }
}

fn compare_optional_decimal(
    result: &mut ComparisonResult,
    path: &str,
    left: Option<Decimal>,
    right: Option<Decimal>,
) {
    match (left, right) {
        (Some(left_value), Some(right_value)) => {
            compare_decimal(result, path, left_value, right_value);
        }

        (Some(left_value), None) => {
            result.push(Difference::new(
                path,
                Some(decimal_string(left_value)),
                None,
            ));
        }

        (None, Some(right_value)) => {
            result.push(Difference::new(
                path,
                None,
                Some(decimal_string(right_value)),
            ));
        }

        (None, None) => {}
    }
}

fn compare_rendered(
    result: &mut ComparisonResult,
    path: &str,
    left: Option<String>,
    right: Option<String>,
) {
    if left != right {
        result.push(Difference::new(path, left, right));
    }
}

fn decimal_string(value: Decimal) -> String {
    value.normalize().to_string()
}

fn money_string(money: &Money) -> String {
    format!(
        "{} {}",
        decimal_string(money.amount,),
        money.currency.as_str()
    )
}

fn line_summary(line: &InvoiceLine) -> String {
    format!("{} — {}", line.description, money_string(&line.net_amount,))
}

fn vat_summary(vat: &VatBreakdown) -> String {
    format!(
        "taxable {}, tax {}",
        money_string(&vat.taxable_amount,),
        money_string(&vat.tax_amount,)
    )
}

fn adjustment_summary(adjustment: &DocumentAdjustment) -> String {
    format!(
        "{} {}",
        adjustment_kind(adjustment.kind,),
        money_string(&adjustment.amount,)
    )
}

fn adjustment_kind(kind: AdjustmentKind) -> &'static str {
    match kind {
        AdjustmentKind::Allowance => "allowance",

        AdjustmentKind::Charge => "charge",
    }
}

fn payment_method(method: PaymentMethod) -> &'static str {
    match method {
        PaymentMethod::BankTransfer => "bank_transfer",

        PaymentMethod::Other => "other",
    }
}
