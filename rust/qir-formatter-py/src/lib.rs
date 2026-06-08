use std::path::Path;

use pyo3::{pyclass, pymethods, pymodule};
use pyo3_stub_gen::{
    Result, StubInfo,
    derive::{gen_stub_pyclass, gen_stub_pymethods},
};

#[gen_stub_pyclass]
#[pyclass(name = "QirLabeledFormatter")]
pub struct QirLabeledFormatter {}

#[gen_stub_pymethods]
#[pymethods]
impl QirLabeledFormatter {
    fn test(&self) -> u64 {
        10
    }
}

#[pymodule]
mod _core {
    #[pymodule_export]
    use super::QirLabeledFormatter;
}

pub fn stub_info() -> Result<StubInfo> {
    let manifest_dir: &Path = env!("CARGO_MANIFEST_DIR").as_ref();
    StubInfo::from_pyproject_toml(manifest_dir.join("../../pyproject.toml"))
}
