use std::collections::HashMap;

use pyo3::prelude::*;

fn qir_type_map(input_type: &String) -> Option<String> {
    match input_type.as_str() {
        "INT" => Some("INT".to_string()),
        "UINT" => Some("INT".to_string()),
        "BOOL" => Some("BOOL".to_string()),
        "FLOAT" => Some("DOUBLE".to_string()),
        "RESULT" => Some("RESULT".to_string()),
        "RESULT_ARRAY" => Some("RESULT_ARRAY".to_string()),
        "QIRARRAY" => Some("ARRAY".to_string()),
        "QIRTUPLE" => Some("TUPLE".to_string()),
        _ => None,
    }
}

#[pyclass]
pub struct QirLabeledFormatter {}

#[derive(FromPyObject, Debug)]
pub enum QShotValType {
    Bool(bool),
    Int(isize),
    Float(f64),
}

#[derive(FromPyObject, Debug)]
pub enum QsysShotItemValue {
    Single(QShotValType),
    Multiple(Vec<QShotValType>),
}

pub type QsysShotItem = (String, QsysShotItemValue);
pub type QsysShot = Vec<QsysShotItem>;
pub type QsysShots = Vec<QsysShot>;

fn results_header(qo: &mut String) {
    qo.push_str("HEADER\tschema_id\tlabeled\n");
    qo.push_str("HEADER\tschema_version\t2.1\n");
}

fn first_shot_header(qo: &mut String, attributes: HashMap<String, Option<String>>) {
    let profile = attributes
        .get("qir_profiles")
        .and_then(|x| x.as_ref())
        .unwrap_or(&"base_profile".to_string())
        .clone();

    let n_qubits = attributes
        .get("required_num_qubits")
        .and_then(|x| x.as_ref())
        .unwrap_or(&"0".to_string())
        .clone();

    let n_results = attributes
        .get("required_num_results")
        .and_then(|x| x.as_ref())
        .unwrap_or(&"0".to_string())
        .clone();

    qo.push_str("START\n");
    qo.push_str("METADATA\tentry_point\n");
    qo.push_str(&format!("METADATA\tqir_profiles\t{profile}\n"));
    qo.push_str(&format!("METADATA\trequired_num_qubits\t{n_qubits}\n"));
    qo.push_str(&format!("METADATA\trequired_num_results\t{n_results}\n"));
}

fn shot_footer(qo: &mut String) {
    qo.push_str("END\t0\n")
}

fn emit(ftype: String, tag: String, val: &QsysShotItemValue) -> Option<String> {
    let qir_type = qir_type_map(&ftype);
    let qir_type = match qir_type {
        Some(q) => q,
        None => return None,
    };

    let value = format_value(&qir_type, val);
    if value.is_none() {
        tracing::warn!(
            raw_type = &ftype,
            qir_type = &qir_type,
            tag = &tag,
            "Skipping malformed QIR output value: {val:?}.",
        );
        return None;
    } else {
        return Some(format!("OUTPUT\t{qir_type}\t{value:?}\t{tag}\n"));
    }
}

fn format_value(type_str: &str, val: &QsysShotItemValue) -> Option<String> {
    if type_str == "RESULT_ARRAY" {
        let val = match val {
            QsysShotItemValue::Single(_) => return None,
            QsysShotItemValue::Multiple(v) => v,
        };

        let mut formatted_bits: Vec<String> = Vec::new();
        for item in val {
            match item {
                QShotValType::Bool(b) => formatted_bits.push((*b as u8).to_string()),
                QShotValType::Int(i) if *i == 0 || *i == 1 => formatted_bits.push(i.to_string()),
                _ => return None,
            }
        }

        return Some(formatted_bits.join(""));
    }

    if type_str == "BOOL" {
        // For BOOLs, the L4 API will always return 0 or 1
        let val_is_truthy = match val {
            QsysShotItemValue::Single(q) => match q {
                QShotValType::Bool(b) => *b,
                QShotValType::Int(i) => *i != 0,
                QShotValType::Float(f) => *f != 0.0,
            },
            QsysShotItemValue::Multiple(_) => return None,
        };
        return Some(val_is_truthy.to_string());
    }

    Some(format!("{val:?}"))
}

fn write_shot(qo: &mut String, shot: &QsysShot) {
    qo.push_str("START\n");
    emit_values_in_shot(qo, shot);
    shot_footer(qo);
}

fn write_first_shot(qo: &mut String, shot: &QsysShot, attributes: HashMap<String, Option<String>>) {
    first_shot_header(qo, attributes);
    emit_values_in_shot(qo, &shot);
    shot_footer(qo);
}

fn emit_values_in_shot(qo: &mut String, shot: &QsysShot) {
    for (tag, value) in shot {
        let fields: Vec<&str> = tag.splitn(3, ":").collect();
        if fields.len() >= 3 && fields[0] == "USER" {
            let out = emit(fields[1].to_string(), fields[2].to_string(), value);
            if let Some(o) = out {
                qo.push_str(&o);
            }
        }
    }
}

#[pymethods]
impl QirLabeledFormatter {
    #[new]
    fn new() -> Self {
        Self {}
    }

    pub fn emit(
        &self,
        qo: &Bound<'_, PyAny>,
        ftype: String,
        tag: String,
        val: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let val: QsysShotItemValue = val.extract::<QsysShotItemValue>()?;
        let out = emit(ftype, tag, &val);
        qo.call_method1("write", (out,))?;
        Ok(())
    }

    pub fn qir_labeled_output(
        &self,
        results: QsysShots,
        attributes: HashMap<String, Option<String>>,
    ) -> String {
        if results.len() == 0 {
            return "".to_string();
        }

        let mut qo = String::new();
        results_header(&mut qo);

        if let [first_shot, results @ ..] = results.as_slice() {
            write_first_shot(&mut qo, first_shot, attributes);
            for shot in results {
                write_shot(&mut qo, shot);
            }
        }

        qo
    }
}

#[pymodule]
fn qir_formatter_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<QirLabeledFormatter>()?;
    Ok(())
}
