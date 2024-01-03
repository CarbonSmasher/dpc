use std::{collections::HashMap, path::PathBuf};

use dpc::{
	ir::IR,
	output::datapack::{Datapack, Function},
	parse::Parser,
	project::ProjectSettings,
	CodegenIRSettings,
};
use pyo3::{exceptions::PyRuntimeError, prelude::*};

/// Formats the sum of two numbers as string.
#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
	Ok((a + b).to_string())
}

/// A Python module implemented in Rust.
#[pymodule]
fn datapack_compiler(_py: Python, m: &PyModule) -> PyResult<()> {
	m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
	m.add_function(wrap_pyfunction!(codegen_ir, m)?)?;
	m.add_function(wrap_pyfunction!(parse_ir, m)?)?;
	m.add_class::<PyIR>()?;
	m.add_class::<PyCodegenIRSettings>()?;
	m.add_class::<PyProjectSettings>()?;
	m.add_class::<PyDatapack>()?;
	Ok(())
}

/// Perform the full compilation routine on IR to convert it to a datapack
#[pyfunction]
fn codegen_ir(
	ir: PyIR,
	project: PyProjectSettings,
	settings: PyCodegenIRSettings,
) -> PyResult<PyDatapack> {
	let pack = dpc::codegen_ir(ir.inner, &project.inner, settings.inner)
		.map_err(|x| PyRuntimeError::new_err(format!("{x:?}")))?;

	Ok(PyDatapack { inner: pack })
}

/// Parse textual IR into actual IR that can be manipulated
#[pyfunction]
fn parse_ir(text: &str) -> PyResult<PyIR> {
	let mut parser = Parser::new();
	parser
		.parse(text)
		.map_err(|x| PyRuntimeError::new_err(format!("{x:?}")))?;
	let ir = parser.finish();
	Ok(PyIR { inner: ir })
}

#[pyclass(name = "IR")]
#[derive(Clone)]
struct PyIR {
	inner: IR,
}

#[pyclass(name = "CodegenIRSettings")]
#[derive(Clone)]
struct PyCodegenIRSettings {
	inner: CodegenIRSettings,
}

#[pymethods]
impl PyCodegenIRSettings {
	#[new]
	fn new() -> Self {
		Self {
			inner: CodegenIRSettings::new(),
		}
	}

	fn debug(&mut self, val: bool) {
		self.inner.debug = val;
	}

	fn debug_functions(&mut self, val: bool) {
		self.inner.debug_functions = val;
	}

	fn ir_passes(&mut self, val: bool) {
		self.inner.ir_passes = val;
	}

	fn mir_passes(&mut self, val: bool) {
		self.inner.mir_passes = val;
	}

	fn lir_passes(&mut self, val: bool) {
		self.inner.lir_passes = val;
	}
}

#[pyclass(name = "ProjectSettings")]
#[derive(Clone)]
struct PyProjectSettings {
	inner: ProjectSettings,
}

#[pymethods]
impl PyProjectSettings {
	#[new]
	fn new(name: String) -> Self {
		Self {
			inner: ProjectSettings::new(name),
		}
	}
}

#[pyclass(name = "Datapack")]
#[derive(Clone)]
struct PyDatapack {
	inner: Datapack,
}

#[pymethods]
impl PyDatapack {
	fn functions(&self) -> HashMap<String, PyFunction> {
		self.inner
			.functions
			.iter()
			.map(|(k, v)| (k.to_string(), PyFunction { inner: v.clone() }))
			.collect()
	}

	fn output(&self, path: &str) -> PyResult<()> {
		let path = PathBuf::from(path);
		self.inner
			.clone()
			.output(&path)
			.map_err(|x| PyRuntimeError::new_err(format!("{x:?}")))?;
		Ok(())
	}
}

#[pyclass(name = "Datapack")]
#[derive(Clone)]
struct PyFunction {
	inner: Function,
}

#[pymethods]
impl PyFunction {
	fn contents(&self) -> Vec<String> {
		self.inner.contents.clone()
	}
}
