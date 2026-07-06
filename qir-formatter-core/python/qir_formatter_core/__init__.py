from typing import TypeAlias, Union

from qir_formatter import QirLabeledFormatter

QShotValType: TypeAlias = Union[int, bool, float]
QsysShotItemValue = QShotValType | list[QShotValType]
# TODO: first element should have max length 256
QsysShotItem = tuple[str, QsysShotItemValue]
QsysShot = list[QsysShotItem]
QsysShots = list[QsysShot]

__all__ = [
    "QShotValType",
    "QsysShotItemValue",
    "QsysShotItem",
    "QsysShot",
    "QsysShots",
    "QirLabeledFormatter",
]
