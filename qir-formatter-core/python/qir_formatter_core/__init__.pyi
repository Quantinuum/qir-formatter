from io import StringIO
from typing import Any

from qir_formatter import QsysShots

class QirLabeledFormatter:
    def emit(self, qo: StringIO, ftype: str, tag: str, val: Any) -> None: ...
    def qir_labeled_output(
        self, results: QsysShots, attributes: dict[str, str | None]
    ) -> str: ...

__all__ = ["QsysShots", "QirLabeledFormatter"]
