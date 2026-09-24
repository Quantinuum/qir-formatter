"""Convert Nexus model of v4 results to QIR spec-compliant results."""

from typing import Annotated, TypeAlias

from pydantic import StringConstraints

from qir_formatter._native import QirLabeledFormatter

QShotValType: TypeAlias = int | bool | float
QsysShotItemValue = QShotValType | list[QShotValType]
QsysShotItem = tuple[
    Annotated[str, StringConstraints(max_length=256)], QsysShotItemValue
]
QsysShot = list[QsysShotItem]
QsysShots = list[QsysShot]

__all__ = [
    "QShotValType",
    "QirLabeledFormatter",
    "QsysShot",
    "QsysShotItem",
    "QsysShotItemValue",
    "QsysShots",
]
