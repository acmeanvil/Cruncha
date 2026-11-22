// SPDX-License-Identifier: GPL-3.0-or-later
//! The prelude of a Rust cell (ADR-0006, spike S5): the cell world's bindings, a minimal
//! `Quantity`, and JSON helpers for the Plain exchange format.

/// The `cell` world's bindings.
pub mod bindings {
    wit_bindgen::generate!({
        world: "cell",
        path: "../../wit",
        pub_export_macro: true,
        export_macro_name: "export_cell",
    });
}

pub use bindings::Guest;

pub use serde_json::{self, Value, json};

/// A number with a unit name, as a calc passes it (SI factor applied by the host).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quantity {
    pub value: f64,
}

/// An input as a float: a Plain number's float or exact value.
pub fn number(inputs: &Value, name: &str) -> Result<f64, String> {
    let v = &inputs[name]["value"];
    match v["type"].as_str() {
        Some("float") => v["value"].as_f64().ok_or_else(|| format!("{name}: not a number")),
        Some("exact") => {
            let n: f64 = v["num"].as_str().and_then(|s| s.parse().ok()).ok_or("bad numerator")?;
            let d: f64 = v["den"].as_str().and_then(|s| s.parse().ok()).ok_or("bad denominator")?;
            Ok(n / d)
        }
        _ => Err(format!("{name} isn't a number")),
    }
}

/// A Plain float.
pub fn float(x: f64) -> Value {
    json!({ "type": "number", "value": { "type": "float", "value": x }, "unit": null })
}
