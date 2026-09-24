"""Convert Nexus model of v4 results to QIR spec-compliant results."""

from typing import Annotated, TypeAlias, Union

from pydantic import StringConstraints

from qir_formatter._native import QirLabeledFormatter

QShotValType: TypeAlias = Union[int, bool, float]
QsysShotItemValue = QShotValType | list[QShotValType]
QsysShotItem = tuple[
    Annotated[str, StringConstraints(max_length=256)], QsysShotItemValue
]
QsysShot = list[QsysShotItem]
QsysShots = list[QsysShot]

__all__ = [
    "QirLabeledFormatter",
    "QShotValType",
    "QsysShot",
    "QsysShotItem",
    "QsysShotItemValue",
    "QsysShots",
]
