//! QIR labeled formatting, with an optional Python compatibility adapter.
//!
//! The Rust API accepts typed shot data and returns QIR 2.1 labeled output:
//!
//! ```
//! use _native::{
//!     QShotValType, QirLabeledFormatter, QirMetadata, QsysShotItemValue, QsysShots,
//! };
//!
//! let results: QsysShots = vec![vec![(
//!     "USER:INT:answer".into(),
//!     QsysShotItemValue::Scalar(QShotValType::Int(42)),
//! )]];
//! let output = QirLabeledFormatter::new().qir_labeled_output(&results, &QirMetadata::new());
//!
//! assert!(output.contains("OUTPUT\tINT\t42\tanswer\n"));
//! ```

#![warn(missing_docs)]

mod labeled_formatter;

pub use labeled_formatter::{
    FormattedValue, QShotValType, QirLabeledFormatter, QirMetadata, QirOutput, QsysShot,
    QsysShotItem, QsysShotItemValue, QsysShots,
};

#[cfg(feature = "python")]
mod python;

#[cfg(feature = "python")]
#[pyo3::pymodule]
mod _native {
    #[pymodule_export]
    use crate::python::PythonQirLabeledFormatter;
}

#[cfg(test)]
mod tests {
    //! Public package surface tests.

    use super::*;

    /// The package should expose a stable top-level import surface.
    #[test]
    fn test_top_level_exports_support_basic_usage() {
        let value: QShotValType = QShotValType::Int(42);
        let item: QsysShotItem = ("USER:INT:answer".into(), QsysShotItemValue::Scalar(value));
        let shot: QsysShot = vec![item];
        let results: QsysShots = vec![shot];

        let output = QirLabeledFormatter::new().qir_labeled_output(&results, &QirMetadata::new());

        assert!(output.contains("OUTPUT\tINT\t42\tanswer\n"));
    }

    /// The main formatter should be directly importable from the package root.
    #[test]
    fn test_top_level_formatter_can_emit_values() {
        let mut output = QirOutput::default();
        QirLabeledFormatter::new().emit(
            &mut output,
            "RESULT_ARRAY",
            Some("bits"),
            &QsysShotItemValue::List(vec![
                QShotValType::Int(1),
                QShotValType::Int(0),
                QShotValType::Int(1),
            ]),
        );

        assert_eq!(output.text, "OUTPUT\tRESULT_ARRAY\t101\tbits\n");
    }

    /// Python package metadata must derive its version from Cargo.
    #[test]
    fn test_python_version_is_dynamic() {
        let pyproject = include_str!("../pyproject.toml");
        let project = pyproject
            .split_once("[project]")
            .expect("pyproject.toml must contain a [project] table")
            .1
            .split("\n[")
            .next()
            .unwrap();

        assert!(
            project
                .lines()
                .any(|line| line.trim() == r#"dynamic = ["version"]"#)
        );
        assert!(
            !project
                .lines()
                .any(|line| line.trim().starts_with("version ="))
        );
    }
}
