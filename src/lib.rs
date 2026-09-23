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
use pyo3::prelude::*;

#[cfg(feature = "python")]
#[pymodule]
fn _native(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<python::PythonQirLabeledFormatter>()?;
    let class = module.getattr("QirLabeledFormatter")?;
    let validators = pyo3::types::PyTuple::new(
        module.py(),
        [class.getattr("_val_tag_type")?, class.getattr("_val_null")?],
    )?;
    class.setattr("val_fns", validators)?;
    Ok(())
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

    /// Cargo and Python package metadata must describe the same release.
    #[test]
    fn test_python_and_rust_versions_match() {
        let pyproject = include_str!("../pyproject.toml");
        let project = pyproject
            .split_once("[project]")
            .expect("pyproject.toml must contain a [project] table")
            .1
            .split("\n[")
            .next()
            .unwrap();
        let python_version = project
            .lines()
            .find_map(|line| {
                line.trim()
                    .strip_prefix("version = \"")
                    .and_then(|value| value.strip_suffix('"'))
            })
            .expect("[project] must contain a quoted version");

        assert_eq!(python_version, env!("CARGO_PKG_VERSION"));
    }
}
