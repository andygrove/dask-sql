use crate::expression::PyExpr;

use datafusion::logical_expr::LogicalPlan;
use datafusion::logical_expr::{logical_plan::Aggregate, Expr};

use crate::sql::exceptions::py_type_err;
use pyo3::prelude::*;

#[pyclass(name = "Aggregate", module = "dask_planner", subclass)]
#[derive(Clone)]
pub struct PyAggregate {
    aggregate: Aggregate,
}

#[pymethods]
impl PyAggregate {
    /// Returns a Vec of the group expressions
    #[pyo3(name = "getGroupSets")]
    pub fn group_expressions(&self) -> PyResult<Vec<PyExpr>> {
        let mut group_exprs: Vec<PyExpr> = Vec::new();
        for expr in &self.aggregate.group_expr {
            group_exprs.push(PyExpr::from(
                expr.clone(),
                Some(vec![self.aggregate.input.clone()]),
            ));
        }
        Ok(group_exprs)
    }

    #[pyo3(name = "getNamedAggCalls")]
    pub fn agg_expressions(&self) -> PyResult<Vec<PyExpr>> {
        let mut agg_exprs: Vec<PyExpr> = Vec::new();
        for expr in &self.aggregate.aggr_expr {
            agg_exprs.push(PyExpr::from(
                expr.clone(),
                Some(vec![self.aggregate.input.clone()]),
            ));
        }
        Ok(agg_exprs)
    }

    #[pyo3(name = "getAggregationFuncName")]
    pub fn agg_func_name(&self, expr: PyExpr) -> PyResult<String> {
        Ok(match expr.expr {
            Expr::AggregateFunction { fun, .. } => fun.to_string(),
            _ => panic!("Encountered a non Aggregate type in agg_func_name"),
        })
    }

    #[pyo3(name = "getArgs")]
    pub fn aggregation_arguments(&self, expr: PyExpr) -> PyResult<Vec<PyExpr>> {
        Ok(match expr.expr {
            Expr::AggregateFunction { fun: _, args, .. } => {
                let mut exprs: Vec<PyExpr> = Vec::new();
                for expr in args {
                    exprs.push(PyExpr {
                        input_plan: Some(vec![self.aggregate.input.clone()]),
                        expr: expr,
                    });
                }
                exprs
            }
            _ => panic!("Encountered a non Aggregate type in agg_func_name"),
        })
    }

    #[pyo3(name = "isDistinct")]
    pub fn distinct(&self, expr: PyExpr) -> PyResult<bool> {
        Ok(match expr.expr {
            Expr::AggregateFunction {
                fun: _,
                args: _,
                distinct,
            } => distinct,
            _ => panic!("Encountered a non Aggregate type in agg_func_name"),
        })
    }
}

impl TryFrom<LogicalPlan> for PyAggregate {
    type Error = PyErr;

    fn try_from(logical_plan: LogicalPlan) -> Result<Self, Self::Error> {
        match logical_plan {
            LogicalPlan::Aggregate(aggregate) => Ok(PyAggregate { aggregate }),
            _ => Err(py_type_err("unexpected plan")),
        }
    }
}