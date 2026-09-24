//! Convert Nexus model of v4 results to QIR spec-compliant results.

use std::collections::HashMap;

/// Scalar values in a shot.
#[derive(Clone, Debug, PartialEq)]
pub enum QShotValType {
    /// A Boolean value.
    Bool(bool),
    /// A signed 64-bit integer, matching the QIR record-output ABI.
    Int(i64),
    /// A double-precision floating-point value.
    Float(f64),
}

/// Invalid inputs are retained so the formatter can skip them.
#[derive(Clone, Debug, PartialEq)]
pub enum QsysShotItemValue {
    /// A single scalar value.
    Scalar(QShotValType),
    /// A sequence of scalar values.
    List(Vec<QShotValType>),
    /// An input that could not be converted to a supported value.
    Invalid,
}

impl QShotValType {
    fn text(&self) -> String {
        match self {
            Self::Bool(value) => if *value { "True" } else { "False" }.to_owned(),
            Self::Int(value) => value.to_string(),
            Self::Float(value) if value.is_nan() => "nan".to_owned(),
            Self::Float(value) if value.is_finite() && value.fract() == 0.0 => format!("{value}.0"),
            Self::Float(value) => value.to_string(),
        }
    }
}

/// A named value from one shot, normally using `USER:<TYPE>:<TAG>`.
pub type QsysShotItem = (String, QsysShotItemValue);
/// All named values in one execution shot.
pub type QsysShot = Vec<QsysShotItem>;
/// A collection of execution shots.
pub type QsysShots = Vec<QsysShot>;
/// Metadata written into the first shot header.
pub type QirMetadata = HashMap<String, String>;

/// A concrete output buffer with a malformed-value count.
#[derive(Default)]
pub struct QirOutput {
    /// Rendered labeled-output text.
    pub text: String,
    /// Number of malformed values skipped while rendering.
    pub malformed: usize,
}

/// The result of applying type-specific formatting to a value.
pub enum FormattedValue {
    /// Preserve the value's ordinary scalar or list representation.
    Original,
    /// Use the supplied formatted text.
    Text(String),
    /// Skip the value as malformed.
    Invalid,
}

/// Formatter for QIR Output Spec results.
#[derive(Default)]
pub struct QirLabeledFormatter;

impl QirLabeledFormatter {
    /// Construct a formatter.
    pub const fn new() -> Self {
        Self
    }

    /// No null tags or null values allowed (empty strings permitted for tags)
    pub fn _val_null(&self, tag: Option<&str>, val: &QsysShotItemValue) -> bool {
        tag.is_some() && !matches!(val, QsysShotItemValue::Invalid)
    }

    /// Tag must be a string
    pub fn _val_tag_type(&self, tag: Option<&str>, _val: &QsysShotItemValue) -> bool {
        tag.is_some()
    }

    /// Ensure the tag and value are valid values
    pub fn validate_tag_and_value(&self, tag: Option<&str>, val: &QsysShotItemValue) -> bool {
        self._val_tag_type(tag, val) && self._val_null(tag, val)
    }

    /// Emit results header.
    pub fn results_header(&self, qo: &mut QirOutput) {
        qo.text.push_str("HEADER\tschema_id\tlabeled\n");
        qo.text.push_str("HEADER\tschema_version\t2.1\n");
    }

    /// Emit opening shot boundary header.
    pub fn first_shot_header(&self, qo: &mut QirOutput, attributes: &QirMetadata) {
        qo.text.push_str("START\n");
        qo.text.push_str("METADATA\tentry_point\n");
        for (key, default) in [
            ("qir_profiles", "base_profile"),
            ("required_num_qubits", "0"),
            ("required_num_results", "0"),
        ] {
            let value = attributes.get(key).map(String::as_str).unwrap_or(default);
            qo.text.push_str(&format!("METADATA\t{key}\t{value}\n"));
        }
    }

    /// Emit closing shot boundary footer.
    pub fn shot_footer(&self, qo: &mut QirOutput) {
        qo.text.push_str("END\t0\n");
    }

    /// Format the value if required
    pub fn format_value(&self, type_str: &str, val: &QsysShotItemValue) -> FormattedValue {
        use QShotValType::{Bool, Float, Int};
        use QsysShotItemValue::{Invalid, List, Scalar};
        match type_str {
            "RESULT_ARRAY" => {
                let List(values) = val else {
                    return FormattedValue::Invalid;
                };
                let bits: Option<String> = values
                    .iter()
                    .map(|value| match value {
                        Bool(false) | Int(0) => Some('0'),
                        Bool(true) | Int(1) => Some('1'),
                        _ => None,
                    })
                    .collect();
                bits.map(FormattedValue::Text)
                    .unwrap_or(FormattedValue::Invalid)
            }
            "BOOL" => {
                // For BOOLs, the L4 API will always return 0 or 1
                let truthy = match val {
                    Scalar(Bool(value)) => *value,
                    Scalar(Int(value)) => *value != 0,
                    Scalar(Float(value)) => *value != 0.0,
                    List(values) => !values.is_empty(),
                    Invalid => false,
                };
                FormattedValue::Text(truthy.to_string())
            }
            _ => FormattedValue::Original,
        }
    }

    /// Emit a value with of the given type and tag.
    pub fn emit(
        &self,
        qo: &mut QirOutput,
        ftype: &str,
        tag: Option<&str>,
        val: &QsysShotItemValue,
    ) {
        // Conversion of internal raw data type to QIR type
        let qir_type = match ftype {
            "INT" | "UINT" => "INT",
            "FLOAT" => "DOUBLE",
            "BOOL" | "RESULT" | "RESULT_ARRAY" => ftype,
            "QIRARRAY" => "ARRAY",
            "QIRTUPLE" => "TUPLE",
            _ => return,
        };
        let text = match self.format_value(qir_type, val) {
            FormattedValue::Text(text) => Some(text),
            FormattedValue::Original => match val {
                QsysShotItemValue::Scalar(value) => Some(value.text()),
                QsysShotItemValue::List(values) => Some(format!(
                    "[{}]",
                    values
                        .iter()
                        .map(QShotValType::text)
                        .collect::<Vec<_>>()
                        .join(", ")
                )),
                QsysShotItemValue::Invalid => None,
            },
            FormattedValue::Invalid => None,
        };
        let Some(text) = text else {
            log::warn!(
                "Skipping malformed QIR output value: raw_type={ftype:?}, qir_type={qir_type:?}, tag={tag:?}, value={val:?}"
            );
            qo.malformed += 1;
            return;
        };
        if self.validate_tag_and_value(tag, val) {
            qo.text
                .push_str(&format!("OUTPUT\t{qir_type}\t{text}\t{}\n", tag.unwrap()));
        }
    }

    /// Given a shot, check the format and emit each user value
    pub fn emit_values_in_shot(&self, qo: &mut QirOutput, shot: &QsysShot) {
        for (name, value) in shot {
            if let Some((ftype, tag)) = name
                .strip_prefix("USER:")
                .and_then(|name| name.split_once(':'))
            {
                self.emit(qo, ftype, Some(tag), value);
            }
        }
    }

    /// Format the user defined output from shots
    pub fn write_shot(&self, qo: &mut QirOutput, shot: &QsysShot) {
        qo.text.push_str("START\n");
        self.emit_values_in_shot(qo, shot);
        self.shot_footer(qo);
    }

    /// Write the first shot, which includes extra metadata
    pub fn write_first_shot(&self, qo: &mut QirOutput, shot: &QsysShot, attributes: &QirMetadata) {
        self.first_shot_header(qo, attributes);
        self.emit_values_in_shot(qo, shot);
        self.shot_footer(qo);
    }

    /// Write all shots, including the schema header and first-shot metadata.
    pub fn write_results(
        &self,
        qo: &mut QirOutput,
        results: &[QsysShot],
        attributes: &QirMetadata,
    ) {
        if let Some((first, remaining)) = results.split_first() {
            self.results_header(qo);
            self.write_first_shot(qo, first, attributes);
            for shot in remaining {
                self.write_shot(qo, shot);
            }
        }
    }

    /// Given a list of results associated with an `n_qubits` job, return
    /// the results in QIR "Labeled" Output Schema format.
    pub fn qir_labeled_output(&self, results: &[QsysShot], attributes: &QirMetadata) -> String {
        let mut output = QirOutput::default();
        self.write_results(&mut output, results, attributes);
        output.text
    }
}

#[cfg(test)]
mod tests {
    //! Test the QIR results formatter

    use super::*;
    use QShotValType::{Bool, Float, Int};
    use QsysShotItemValue::{Invalid, List, Scalar};

    fn shot(items: impl IntoIterator<Item = (&'static str, QsysShotItemValue)>) -> QsysShot {
        items
            .into_iter()
            .map(|(tag, value)| (tag.into(), value))
            .collect()
    }

    /// Test raw data types that are rendered to QIR output.
    #[test]
    #[allow(clippy::approx_constant)] // Preserve the Python test's literal input.
    fn test_formatting() {
        for (ftype, tag, value, expected) in [
            ("INT", "i0", Scalar(Int(42)), "OUTPUT\tINT\t42\ti0\n"),
            ("UINT", "ui0", Scalar(Int(99)), "OUTPUT\tINT\t99\tui0\n"),
            (
                "FLOAT",
                "f0",
                Scalar(Float(3.1415926)),
                "OUTPUT\tDOUBLE\t3.1415926\tf0\n",
            ),
            ("BOOL", "b0", Scalar(Int(0)), "OUTPUT\tBOOL\tfalse\tb0\n"),
            ("RESULT", "r0", Scalar(Int(0)), "OUTPUT\tRESULT\t0\tr0\n"),
            (
                "RESULT_ARRAY",
                "ra0",
                List(vec![Int(1), Int(0), Int(1)]),
                "OUTPUT\tRESULT_ARRAY\t101\tra0\n",
            ),
            ("QIRARRAY", "a0", Scalar(Int(4)), "OUTPUT\tARRAY\t4\ta0\n"),
            ("QIRTUPLE", "t0", Scalar(Int(13)), "OUTPUT\tTUPLE\t13\tt0\n"),
        ] {
            let mut output = QirOutput::default();
            QirLabeledFormatter::new().emit(&mut output, ftype, Some(tag), &value);
            assert_eq!(output.text, expected, "{ftype}");
        }
    }

    /// Test complete formatting of a valid list of raw results.
    #[test]
    fn test_result_list() {
        // 5 shots of a variety of results
        let results = vec![
            shot([
                ("USER:INT:i1", Scalar(Int(42))),
                ("USER:BOOL:b1", Scalar(Int(1))),
            ]),
            shot([
                ("USER:RESULT:r1", Scalar(Int(99))),
                ("USER:INT:i2", Scalar(Int(-54321))),
                ("USER:FLOAT:f1", Scalar(Float(std::f64::consts::E))),
            ]),
            shot([
                ("USER:FLOAT:φ", Scalar(Float(std::f64::consts::PI))),
                ("USER:INT:large", Scalar(Int(i64::MAX))),
                ("USER:INT:neg", Scalar(Int(i64::MIN))),
            ]),
            shot([(
                "USER:RESULT_ARRAY:results",
                List(vec![Int(1), Int(0), Int(1), Int(1)]),
            )]),
            shot([
                ("USER:QIRARRAY:0_a", Scalar(Int(2))),
                ("USER:QIRTUPLE:1_a0t", Scalar(Int(2))),
                ("USER:INT:2_a0t0i", Scalar(Int(42))),
                ("USER:RESULT:3_a0t1r", Scalar(Int(0))),
                ("USER:QIRTUPLE:4_a1t", Scalar(Int(2))),
                ("USER:INT:5_a1t0i", Scalar(Int(33))),
                ("USER:RESULT:6_a1t1r", Scalar(Int(1))),
            ]),
        ];
        let attributes = QirMetadata::from([
            ("qir_profiles".into(), "base_profile".into()),
            ("required_num_qubits".into(), "9".into()),
            ("required_num_results".into(), "9".into()),
        ]);
        assert_eq!(
            QirLabeledFormatter::new().qir_labeled_output(&results, &attributes),
            include_str!("tests/data/good1.output")
        );
    }

    /// Test complete formatting of a valid list of raw results, with
    /// some fields that are not rendered for QIR output.
    #[test]
    fn test_full_raw_result_list() {
        let results = vec![
            shot([
                ("MEAS:BOOL:", Scalar(Int(1))),
                ("MEAS:BOOL:", Scalar(Int(1))),
                ("MEAS:BOOL:", Scalar(Int(0))),
                ("USER:INT:reg1", Scalar(Int(3))),
                ("USER:FLOAT:reg2", Scalar(Float(1.43))),
                ("USER:INT:reg:3", Scalar(Int(1))),
                ("METRICS:INT:SQ", Scalar(Int(12))),
                ("METRICS:INT:TQ", Scalar(Int(15))),
                ("METRICS:INT:SPAM", Scalar(Int(8))),
                ("METRICS:INT:NumQubits", Scalar(Int(4))),
                ("METRICS:FLOAT:ShotTime", Scalar(Float(0.0194))),
            ]),
            shot([
                ("MEAS:BOOL:", Scalar(Int(0))),
                ("MEAS:BOOL:", Scalar(Int(1))),
                ("MEAS:BOOL:", Scalar(Int(0))),
                ("MEAS:BOOL:", Scalar(Int(1))),
                ("USER:INT:reg1", Scalar(Int(9))),
                ("MEAS:BOOL:", Scalar(Int(1))),
                ("MEAS:BOOL:", Scalar(Int(0))),
                ("USER:INTARR:reg3", List(vec![Int(5), Int(12)])),
                ("METRICS:INT:SQ", Scalar(Int(15))),
                ("METRICS:INT:TQ", Scalar(Int(19))),
                ("METRICS:INT:SPAM", Scalar(Int(12))),
                ("METRICS:FLOAT:ShotTime", Scalar(Float(0.0394))),
            ]),
            shot([
                ("MEAS:BOOL:", Scalar(Int(1))),
                ("MEAS:BOOL:", Scalar(Int(1))),
                ("MEAS:BOOL:", Scalar(Int(1))),
                ("MEAS:BOOL:", Scalar(Int(1))),
                ("USER:INT:reg1", Scalar(Int(-2))),
                (
                    "EXIT:INT:Unexpected results indicating deep flaw/exit",
                    Scalar(Int(1000)),
                ),
                ("METRICS:INT:SQ", Scalar(Int(12))),
                ("METRICS:INT:TQ", Scalar(Int(15))),
                ("METRICS:INT:SPAM", Scalar(Int(8))),
                ("METRICS:FLOAT:ShotTime", Scalar(Float(0.0194))),
            ]),
        ];
        let attributes = QirMetadata::from([
            ("qir_profiles".into(), "base_profile".into()),
            ("required_num_qubits".into(), "9".into()),
            ("required_num_results".into(), "3".into()),
        ]);
        for (attributes, expected) in [
            (attributes, include_str!("tests/data/full1.output")),
            (
                QirMetadata::new(),
                include_str!("tests/data/missing_attributes.output"),
            ),
        ] {
            assert_eq!(
                QirLabeledFormatter::new().qir_labeled_output(&results, &attributes),
                expected
            );
        }
    }

    /// Test valid raw data types are ignored.
    #[test]
    fn test_ignored_types() {
        for ftype in ["BOOLARR", "INTARR", "UINTARR", "FLOATARR"] {
            let mut output = QirOutput::default();
            QirLabeledFormatter::new().emit(
                &mut output,
                ftype,
                Some("aggr"),
                &List(vec![Int(1), Int(1), Int(0)]),
            );
            assert!(output.text.is_empty(), "{ftype}");
            assert_eq!(output.malformed, 0);
        }
    }

    /// Test invalid raw data type.
    #[test]
    fn test_undefined_type() {
        let mut output = QirOutput::default();
        QirLabeledFormatter::new().emit(
            &mut output,
            "UINT32",
            Some("aggr"),
            &List(vec![Int(1), Int(1), Int(0)]),
        );
        assert!(output.text.is_empty());
    }

    /// Test improperly formatted raw data.
    #[test]
    fn test_malformed() {
        // Non-string tags and None values are represented by None and Invalid at the Rust boundary.
        for (ftype, tag, value) in [
            ("INT", None, Scalar(Int(0))),
            ("INT", Some("syndrome0"), Invalid),
            ("BOOL", None, Invalid),
            ("FLOAT", None, Scalar(Int(99))),
            ("", None, Invalid),
        ] {
            let mut output = QirOutput::default();
            QirLabeledFormatter::new().emit(&mut output, ftype, tag, &value);
            assert!(output.text.is_empty(), "{ftype}");
        }
    }

    /// Test complete formatting of a valid list of raw results, with
    /// some malformed entries.
    #[test]
    fn test_malformed_raw_result_list() {
        // Invalid Python entry shapes are filtered by the adapter; test the remaining typed entries.
        let results = vec![shot([
            ("USER:INT:reg1", Scalar(Int(3))),
            ("USER::reg3", Scalar(Float(1.414))),
            ("USER:RESULT:r1", Invalid),
            ("USER:FLOAT:reg2", Scalar(Float(1.43))),
            (":INT:i0", Scalar(Int(1))),
            ("::i0", Scalar(Int(1))),
            (":::", Scalar(Int(1))),
            ("::", Scalar(Int(0))),
            (":", Scalar(Int(0))),
            (":", Invalid),
            (":", Invalid),
            ("f", Invalid),
            ("f", Invalid),
            ("USER:BOOL:", Scalar(Int(1))),
        ])];
        let attributes = QirMetadata::from([
            ("qir_profiles".into(), "base_profile".into()),
            ("required_num_qubits".into(), "9".into()),
            ("required_num_results".into(), "9".into()),
        ]);
        assert_eq!(
            QirLabeledFormatter::new().qir_labeled_output(&results, &attributes),
            include_str!("tests/data/malformed1.output")
        );
    }

    /// Test empty tag submission produces output.
    #[test]
    fn test_empty_tag_submission() {
        let results = vec![shot([
            ("USER:QIRARRAY:", Scalar(Int(2))),
            ("USER:QIRTUPLE:", Scalar(Int(2))),
        ])];
        let output = QirLabeledFormatter::new().qir_labeled_output(&results, &QirMetadata::new());
        // assert that the output still shows with ARRAY and TUPLE types
        assert!(output.contains("OUTPUT\tARRAY\t2\t\n"));
        assert!(output.contains("OUTPUT\tTUPLE\t2\t\n"));
    }

    /// Result arrays are emitted as a single binary string record.
    #[test]
    fn test_result_array_formatting() {
        for (value, expected) in [
            (vec![], ""),
            (vec![Bool(false), Bool(true), Bool(false)], "010"),
        ] {
            let mut output = QirOutput::default();
            QirLabeledFormatter::new().emit(
                &mut output,
                "RESULT_ARRAY",
                Some("results"),
                &List(value),
            );
            assert_eq!(
                output.text,
                format!("OUTPUT\tRESULT_ARRAY\t{expected}\tresults\n")
            );
        }
    }

    /// Malformed result arrays should not raise and should be ignored.
    #[test]
    fn test_malformed_result_array_is_ignored() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        struct Logger(AtomicUsize);
        impl log::Log for Logger {
            fn enabled(&self, _: &log::Metadata<'_>) -> bool {
                true
            }
            fn log(&self, record: &log::Record<'_>) {
                let message = record.args().to_string();
                // Other tests can emit warnings concurrently for different tags.
                if message.contains("tag=Some(\"results\")") {
                    assert_eq!(record.level(), log::Level::Warn);
                    assert!(message.starts_with("Skipping malformed QIR output value"));
                    self.0.fetch_add(1, Ordering::Relaxed);
                }
            }
            fn flush(&self) {}
        }
        static LOGGER: Logger = Logger(AtomicUsize::new(0));
        log::set_logger(&LOGGER).unwrap();
        log::set_max_level(log::LevelFilter::Warn);

        // None, 1, "010", [2], ["x"], and (1, 0, 1) after conversion to Rust values.
        for value in [
            Invalid,
            Scalar(Int(1)),
            Invalid,
            List(vec![Int(2)]),
            Invalid,
            Invalid,
        ] {
            let mut output = QirOutput::default();
            QirLabeledFormatter::new().emit(&mut output, "RESULT_ARRAY", Some("results"), &value);
            assert!(output.text.is_empty());
            assert_eq!(output.malformed, 1);
        }
        assert_eq!(LOGGER.0.load(Ordering::Relaxed), 6);
    }

    #[test]
    fn formats_scalar_types_and_empty_tags() {
        let formatter = QirLabeledFormatter::new();
        for (raw, value, expected) in [
            ("INT", Int(42), "INT\t42"),
            ("UINT", Int(99), "INT\t99"),
            ("FLOAT", Float(1.5), "DOUBLE\t1.5"),
            ("BOOL", Int(0), "BOOL\tfalse"),
            ("RESULT", Int(1), "RESULT\t1"),
            ("QIRARRAY", Int(4), "ARRAY\t4"),
            ("QIRTUPLE", Int(2), "TUPLE\t2"),
        ] {
            let mut output = QirOutput::default();
            formatter.emit(&mut output, raw, Some(""), &Scalar(value));
            assert_eq!(output.text, format!("OUTPUT\t{expected}\t\n"));
            assert_eq!(output.malformed, 0);
        }
    }

    #[test]
    fn result_arrays_and_malformed_inputs() {
        let formatter = QirLabeledFormatter::new();
        let mut output = QirOutput::default();
        formatter.emit(
            &mut output,
            "RESULT_ARRAY",
            Some("bits"),
            &List(vec![Bool(true), Int(0), Int(1)]),
        );
        assert_eq!(output.text, "OUTPUT\tRESULT_ARRAY\t101\tbits\n");
        for value in [
            Invalid,
            Scalar(Int(1)),
            List(vec![Int(2)]),
            List(vec![Float(1.0)]),
        ] {
            formatter.emit(&mut output, "RESULT_ARRAY", Some("bits"), &value);
        }
        assert_eq!(output.malformed, 4);
        formatter.emit(&mut output, "unknown", Some("tag"), &Invalid);
        formatter.emit(&mut output, "INT", None, &Scalar(Int(1)));
        assert_eq!(output.malformed, 4);
        assert!(!formatter.validate_tag_and_value(Some("tag"), &Invalid));
    }

    #[test]
    fn shot_methods_compose() {
        let formatter = QirLabeledFormatter::new();
        let shots = vec![vec![("USER:INT:a".into(), Scalar(Int(42)))], vec![]];
        let attributes = QirMetadata::new();
        let mut output = QirOutput::default();
        formatter.results_header(&mut output);
        formatter.write_first_shot(&mut output, &shots[0], &attributes);
        formatter.write_shot(&mut output, &shots[1]);
        assert_eq!(
            formatter.qir_labeled_output(&shots, &attributes),
            output.text
        );
        assert_eq!(formatter.qir_labeled_output(&[], &attributes), "");
        assert!(output.text.ends_with("END\t0\nSTART\nEND\t0\n"));
    }

    #[test]
    fn matches_shared_malformed_fixture() {
        let shots = vec![vec![
            ("USER:INT:reg1".into(), Scalar(Int(3))),
            ("USER::reg3".into(), Scalar(Float(1.414))),
            ("USER:RESULT:r1".into(), Invalid),
            ("USER:FLOAT:reg2".into(), Scalar(Float(1.43))),
            (":INT:i0".into(), Scalar(Int(1))),
            ("USER:BOOL:".into(), Scalar(Int(1))),
        ]];
        let attributes = QirMetadata::from([
            ("required_num_qubits".into(), "9".into()),
            ("required_num_results".into(), "9".into()),
        ]);
        assert_eq!(
            QirLabeledFormatter::new().qir_labeled_output(&shots, &attributes),
            include_str!("tests/data/malformed1.output")
        );
    }
}
