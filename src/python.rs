//! Conversion and I/O only; the formatter lives in labeled_formatter.rs.
use crate::{
    FormattedValue, QShotValType, QirLabeledFormatter, QirMetadata, QirOutput, QsysShot,
    QsysShotItemValue,
};
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyDict, PyFloat, PyInt, PyList, PyTuple};

/// Convert a Python scalar, checking bool before int because bool is an int subclass.
fn scalar(value: &Bound<'_, PyAny>) -> Option<QShotValType> {
    if value.is_instance_of::<PyBool>() {
        value.extract().ok().map(QShotValType::Bool)
    } else if value.is_instance_of::<PyInt>() {
        value.extract().ok().map(QShotValType::Int)
    } else if value.is_instance_of::<PyFloat>() {
        value.extract().ok().map(QShotValType::Float)
    } else {
        None
    }
}

/// Convert a scalar or list of scalars; mark unsupported values as invalid.
fn value(input: &Bound<'_, PyAny>) -> QsysShotItemValue {
    if let Some(value) = scalar(input) {
        QsysShotItemValue::Scalar(value)
    } else if let Ok(values) = input.cast::<PyList>() {
        values
            .iter()
            .map(|item| scalar(&item))
            .collect::<Option<Vec<_>>>()
            .map(QsysShotItemValue::List)
            .unwrap_or(QsysShotItemValue::Invalid)
    } else {
        QsysShotItemValue::Invalid
    }
}

/// Represent whether a value is None without checking its type, for validation.
fn presence(input: &Bound<'_, PyAny>) -> QsysShotItemValue {
    if input.is_none() {
        QsysShotItemValue::Invalid
    } else {
        QsysShotItemValue::Scalar(QShotValType::Bool(true))
    }
}

/// Convert shot entries, skipping None, short entries, and non-string tags.
fn shot(input: &Bound<'_, PyAny>) -> PyResult<QsysShot> {
    let mut shot = Vec::new();
    for item in input.try_iter()? {
        let item = item?;
        if item.is_none() || item.len()? < 2 {
            continue;
        }
        if let Ok(name) = item.get_item(0)?.extract::<String>() {
            shot.push((name, value(&item.get_item(1)?)));
        }
    }
    Ok(shot)
}

/// Convert metadata to string keys and Python string representations of values.
fn metadata(input: &Bound<'_, PyAny>) -> PyResult<QirMetadata> {
    input
        .cast::<PyDict>()?
        .iter()
        .map(|(key, value)| Ok((key.extract()?, value.str()?.extract()?)))
        .collect()
}

/// Emit the Python warning for each skipped malformed value and return the output.
fn finish(py: Python<'_>, output: QirOutput) -> PyResult<String> {
    if output.malformed > 0 {
        let logger = py
            .import("logging")?
            .call_method1("getLogger", ("qir_formatter.labeled_formatter",))?;
        for _ in 0..output.malformed {
            logger.call_method1("warning", ("Skipping malformed QIR output value",))?;
        }
    }
    Ok(output.text)
}

/// Render into a Rust buffer, emit Python warnings, and write any text to the Python writer.
fn write(qo: &Bound<'_, PyAny>, render: impl FnOnce(&mut QirOutput)) -> PyResult<()> {
    let mut output = QirOutput::default();
    render(&mut output);
    let text = finish(qo.py(), output)?;
    if !text.is_empty() {
        qo.call_method1("write", (text,))?;
    }
    Ok(())
}

/// Formatter for QIR Output Spec results.
#[pyclass(
    name = "QirLabeledFormatter",
    module = "qir_formatter.labeled_formatter"
)]
pub(crate) struct PythonQirLabeledFormatter(QirLabeledFormatter);

#[pymethods]
impl PythonQirLabeledFormatter {
    #[new]
    fn new() -> Self {
        Self(QirLabeledFormatter::new())
    }

    #[classattr]
    fn val_fns(py: Python<'_>) -> PyResult<Py<PyTuple>> {
        let class = py.get_type::<Self>();
        PyTuple::new(
            py,
            [class.getattr("_val_tag_type")?, class.getattr("_val_null")?],
        )
        .map(Bound::unbind)
    }

    /// No null tags or null values allowed (empty strings permitted for tags)
    #[pyo3(signature = (tag: "str", val))]
    fn _val_null(&self, tag: &Bound<'_, PyAny>, val: &Bound<'_, PyAny>) -> bool {
        self.0
            ._val_null((!tag.is_none()).then_some(""), &presence(val))
    }

    /// Tag must be a string
    #[pyo3(signature = (tag: "str", _val))]
    fn _val_tag_type(&self, tag: &Bound<'_, PyAny>, _val: &Bound<'_, PyAny>) -> bool {
        self.0
            ._val_tag_type(tag.extract::<&str>().ok(), &presence(_val))
    }

    /// Ensure the tag and value are valid values
    #[pyo3(signature = (tag: "str", val))]
    fn validate_tag_and_value(&self, tag: &Bound<'_, PyAny>, val: &Bound<'_, PyAny>) -> bool {
        self.0
            .validate_tag_and_value(tag.extract::<&str>().ok(), &presence(val))
    }

    /// Emit results header.
    fn results_header(&self, qo: &Bound<'_, PyAny>) -> PyResult<()> {
        write(qo, |out| self.0.results_header(out))
    }

    /// Emit opening shot boundary header.
    #[pyo3(signature = (qo, attributes: "dict[str, str | None]"))]
    fn first_shot_header(
        &self,
        qo: &Bound<'_, PyAny>,
        attributes: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let attributes = metadata(attributes)?;
        write(qo, |out| self.0.first_shot_header(out, &attributes))
    }

    /// Emit closing shot boundary footer.
    fn shot_footer(&self, qo: &Bound<'_, PyAny>) -> PyResult<()> {
        write(qo, |out| self.0.shot_footer(out))
    }

    /// Emit a value with of the given type and tag.
    #[pyo3(signature = (qo, ftype: "str", tag: "str", val))]
    fn emit(
        &self,
        qo: &Bound<'_, PyAny>,
        ftype: &Bound<'_, PyAny>,
        tag: &Bound<'_, PyAny>,
        val: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        write(qo, |out| {
            self.0.emit(
                out,
                ftype.extract::<&str>().unwrap_or(""),
                tag.extract::<&str>().ok(),
                &value(val),
            )
        })
    }

    /// Format the value if required
    #[pyo3(signature = (type_str: "str", val))]
    fn format_value(&self, type_str: &str, val: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        match self.0.format_value(type_str, &value(val)) {
            FormattedValue::Original => Ok(val.clone().unbind()),
            FormattedValue::Text(text) => Ok(text.into_pyobject(val.py())?.into_any().unbind()),
            FormattedValue::Invalid => Ok(val.py().None()),
        }
    }

    /// Format the user defined output from shots
    fn write_shot(&self, qo: &Bound<'_, PyAny>, shot: &Bound<'_, PyAny>) -> PyResult<()> {
        let shot = self::shot(shot)?;
        write(qo, |out| self.0.write_shot(out, &shot))
    }

    /// Write the first shot, which includes extra metadata
    #[pyo3(signature = (qo, shot, attributes: "dict[str, str | None]"))]
    fn write_first_shot(
        &self,
        qo: &Bound<'_, PyAny>,
        shot: &Bound<'_, PyAny>,
        attributes: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let shot = self::shot(shot)?;
        let attributes = metadata(attributes)?;
        write(qo, |out| self.0.write_first_shot(out, &shot, &attributes))
    }

    /// Given a shot, check the format and emit each user value
    fn emit_values_in_shot(&self, qo: &Bound<'_, PyAny>, shot: &Bound<'_, PyAny>) -> PyResult<()> {
        let shot = self::shot(shot)?;
        write(qo, |out| self.0.emit_values_in_shot(out, &shot))
    }

    /// Given a list of results associated with an `n_qubits` job, return
    /// the results in QIR "Labeled" Output Schema format.
    #[pyo3(signature = (results, attributes: "dict[str, str | None]"))]
    fn qir_labeled_output(
        &self,
        results: &Bound<'_, PyAny>,
        attributes: &Bound<'_, PyAny>,
    ) -> PyResult<String> {
        if results.len()? == 0 {
            return Ok(String::new());
        }
        let shots = results
            .try_iter()?
            .map(|item| shot(&item?))
            .collect::<PyResult<Vec<_>>>()?;
        let mut output = QirOutput::default();
        self.0
            .write_results(&mut output, &shots, &metadata(attributes)?);
        finish(results.py(), output)
    }
}
