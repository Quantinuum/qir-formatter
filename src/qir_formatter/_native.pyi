from typing import Any, Final, final

@final
class QirLabeledFormatter:
    """
    Formatter for QIR Output Spec results.
    """
    val_fns: Final[tuple]
    def __new__(cls, /) -> QirLabeledFormatter: ...
    def _val_null(self, /, tag: "str", val: Any) -> bool:
        """
        No null tags or null values allowed (empty strings permitted for tags)
        """
    def _val_tag_type(self, /, tag: "str", _val: Any) -> bool:
        """
        Tag must be a string
        """
    def emit(self, /, qo: Any, ftype: "str", tag: "str", val: Any) -> None:
        """
        Emit a value with of the given type and tag.
        """
    def emit_values_in_shot(self, /, qo: Any, shot: Any) -> None:
        """
        Given a shot, check the format and emit each user value
        """
    def first_shot_header(self, /, qo: Any, attributes: "dict[str, str | None]") -> None:
        """
        Emit opening shot boundary header.
        """
    def format_value(self, /, type_str: "str", val: Any) -> Any:
        """
        Format the value if required
        """
    def qir_labeled_output(self, /, results: Any, attributes: "dict[str, str | None]") -> str:
        """
        Given a list of results associated with an `n_qubits` job, return
        the results in QIR "Labeled" Output Schema format.
        """
    def results_header(self, /, qo: Any) -> None:
        """
        Emit results header.
        """
    def shot_footer(self, /, qo: Any) -> None:
        """
        Emit closing shot boundary footer.
        """
    def validate_tag_and_value(self, /, tag: "str", val: Any) -> bool:
        """
        Ensure the tag and value are valid values
        """
    def write_first_shot(self, /, qo: Any, shot: Any, attributes: "dict[str, str | None]") -> None:
        """
        Write the first shot, which includes extra metadata
        """
    def write_shot(self, /, qo: Any, shot: Any) -> None:
        """
        Format the user defined output from shots
        """
