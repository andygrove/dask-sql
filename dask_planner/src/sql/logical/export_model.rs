use crate::sql::{exceptions::py_type_err, logical, parser_utils::DaskParserUtils};
use pyo3::prelude::*;

use datafusion_expr::{logical_plan::UserDefinedLogicalNode, Expr, LogicalPlan};
use datafusion_sql::sqlparser::ast::Expr as SqlParserExpr;

use fmt::Debug;
use std::{any::Any, collections::HashMap, fmt, sync::Arc};

use datafusion_common::{DFSchema, DFSchemaRef};

#[derive(Clone)]
pub struct ExportModelPlanNode {
    pub schema: DFSchemaRef,
    pub model_name: String,
    pub with_options: Vec<SqlParserExpr>,
}

impl Debug for ExportModelPlanNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt_for_explain(f)
    }
}

impl UserDefinedLogicalNode for ExportModelPlanNode {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn inputs(&self) -> Vec<&LogicalPlan> {
        vec![]
    }

    fn schema(&self) -> &DFSchemaRef {
        &self.schema
    }

    fn expressions(&self) -> Vec<Expr> {
        // there is no need to expose any expressions here since DataFusion would
        // not be able to do anything with expressions that are specific to
        // EXPORT MODEL
        vec![]
    }

    fn fmt_for_explain(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ExportModel: model_name={}", self.model_name)
    }

    fn from_template(
        &self,
        _exprs: &[Expr],
        inputs: &[LogicalPlan],
    ) -> Arc<dyn UserDefinedLogicalNode> {
        assert_eq!(inputs.len(), 0, "input size inconsistent");
        Arc::new(ExportModelPlanNode {
            schema: Arc::new(DFSchema::empty()),
            model_name: self.model_name.clone(),
            with_options: self.with_options.clone(),
        })
    }
}

#[pyclass(name = "ExportModel", module = "dask_planner", subclass)]
pub struct PyExportModel {
    pub(crate) export_model: ExportModelPlanNode,
}

#[pymethods]
impl PyExportModel {
    #[pyo3(name = "getModelName")]
    fn get_model_name(&self) -> PyResult<String> {
        Ok(self.export_model.model_name.clone())
    }

    #[pyo3(name = "getSQLWithOptions")]
    fn sql_with_options(&self) -> PyResult<HashMap<String, String>> {
        let mut options: HashMap<String, String> = HashMap::new();
        for elem in &self.export_model.with_options {
            match elem {
                SqlParserExpr::BinaryOp { left, op: _, right } => {
                    options.insert(
                        DaskParserUtils::str_from_expr(*left.clone()),
                        DaskParserUtils::str_from_expr(*right.clone()),
                    );
                }
                _ => {
                    return Err(py_type_err(
                        "Encountered non SqlParserExpr::BinaryOp expression, with arguments can only be of Key/Value pair types"));
                }
            }
        }
        Ok(options)
    }
}

impl TryFrom<logical::LogicalPlan> for PyExportModel {
    type Error = PyErr;

    fn try_from(logical_plan: logical::LogicalPlan) -> Result<Self, Self::Error> {
        match logical_plan {
            logical::LogicalPlan::Extension(extension) => {
                if let Some(ext) = extension
                    .node
                    .as_any()
                    .downcast_ref::<ExportModelPlanNode>()
                {
                    Ok(PyExportModel {
                        export_model: ext.clone(),
                    })
                } else {
                    Err(py_type_err("unexpected plan"))
                }
            }
            _ => Err(py_type_err("unexpected plan")),
        }
    }
}
