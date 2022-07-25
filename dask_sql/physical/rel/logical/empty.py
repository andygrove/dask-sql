import logging
from typing import TYPE_CHECKING

import dask.dataframe as dd
import pandas as pd

from dask_sql.datacontainer import ColumnContainer, DataContainer
from dask_sql.physical.rel.base import BaseRelPlugin

if TYPE_CHECKING:
    import dask_sql
    from dask_planner.rust import LogicalPlan

logger = logging.getLogger(__name__)


class DaskEmptyRelationPlugin(BaseRelPlugin):
    """
    When a SQL query does not contain a target table, this plugin is invoked to
    create an empty DataFrame that the remaining expressions can operate against.
    """

    class_name = "EmptyRelation"

    def convert(self, rel: "LogicalPlan", context: "dask_sql.Context") -> DataContainer:
        return DataContainer(
            dd.from_pandas(pd.DataFrame([0], columns=["_empty"]), npartitions=1),
            ColumnContainer([]),
        )