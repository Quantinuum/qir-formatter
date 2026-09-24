"""Type declarations for the private native formatter extension."""

from io import StringIO
from typing import Any

class QirLabeledFormatter:
    """Formatter for QIR Output Spec results."""

    val_fns: tuple[Any, ...]

    def _val_null(self, tag: str, val: Any) -> bool:
        """No null tags or null values allowed (empty strings permitted for tags)"""
        ...
    def _val_tag_type(self, tag: str, _val: Any) -> bool:
        """Tag must be a string"""
        ...
    def validate_tag_and_value(self, tag: str, val: Any) -> bool:
        """Ensure the tag and value are valid values"""
        ...
    def results_header(self, qo: StringIO) -> None:
        """Emit results header."""
        ...
    def first_shot_header(
        self, qo: StringIO, attributes: dict[str, str | None]
    ) -> None:
        """Emit opening shot boundary header."""
        ...
    def shot_footer(self, qo: StringIO) -> None:
        """Emit closing shot boundary footer."""
        ...
    def emit(self, qo: StringIO, ftype: str, tag: str, val: Any) -> None:
        """Emit a value with of the given type and tag."""
        ...
    def format_value(self, type_str: str, val: Any) -> Any:
        """Format the value if required"""
        ...
    def write_shot(self, qo: StringIO, shot: Any) -> None:
        """Format the user defined output from shots"""
        ...
    def write_first_shot(
        self,
        qo: StringIO,
        shot: Any,
        attributes: dict[str, str | None],
    ) -> None:
        """Write the first shot, which includes extra metadata"""
        ...
    def emit_values_in_shot(self, qo: StringIO, shot: Any) -> None:
        """Given a shot, check the format and emit each user value"""
        ...
    def qir_labeled_output(
        self, results: Any, attributes: dict[str, str | None]
    ) -> str:
        """
        Given a list of results associated with an `n_qubits` job, return
        the results in QIR "Labeled" Output Schema format.
        """
        ...
