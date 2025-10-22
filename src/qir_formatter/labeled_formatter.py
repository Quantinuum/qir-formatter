"""Convert Nexus model of v4 results to QIR spec-compliant results."""

from io import StringIO
from typing import Annotated, Dict, TypeAlias, Union

from pydantic import StringConstraints

QShotValType: TypeAlias = Union[int, bool, float]
QsysShotItemValue = QShotValType | list[QShotValType]
QsysShotItem = tuple[
    Annotated[str, StringConstraints(max_length=256)], QsysShotItemValue
]
QsysShot = list[QsysShotItem]
QsysShots = list[QsysShot]

# Conversion of internal raw data type to QIR type
QIR_TYPE_MAP = {
    "INT": "INT",
    "UINT": "INT",
    "BOOL": "BOOL",
    "FLOAT": "DOUBLE",
    "RESULT": "RESULT",
    "QIRARRAY": "ARRAY",
    "QIRTUPLE": "TUPLE",
}


class QirLabeledFormatter:
    """Formatter for QIR Output Spec results."""

    def _val_null(self, tag: str, val) -> bool:
        """No null tags or null values allowed (empty strings permitted for tags)"""
        return (tag is not None) and (val is not None)

    def _val_tag_type(self, tag: str, _val) -> bool:
        """Tag must be a string"""
        return isinstance(tag, str)

    val_fns = (
        _val_tag_type,
        _val_null,
    )

    def validate_tag_and_value(self, tag: str, val) -> bool:
        """Ensure the tag and value are valid values"""
        for valfunc in self.val_fns:
            if not valfunc(self, tag, val):
                return False

        return True

    def results_header(self, qo: StringIO):
        """Emit results header."""
        qo.write("HEADER\tschema_id\tlabeled\n")
        qo.write("HEADER\tschema_version\t1.0\n")

    def first_shot_header(self, qo: StringIO, attributes: Dict[str, str | None]):
        """Emit opening shot boundary header."""
        qo.write("START\n")
        qo.write("METADATA\tentry_point\n")
        qo.write(
            f"METADATA\tqir_profiles\t{attributes.get('qir_profiles', 'base_profile')}\n"
        )
        qo.write(
            f"METADATA\trequired_num_qubits\t{attributes.get('required_num_qubits', 0)}\n"
        )
        qo.write(
            f"METADATA\trequired_num_results\t{attributes.get('required_num_results', 0)}\n"
        )

    def shot_footer(self, qo: StringIO):
        """Emit closing shot boundary footer."""
        qo.write("END\t0\n")

    def emit(self, qo: StringIO, ftype: str, tag: str, val):
        """Emit a value with of the given type and tag."""
        qir_type = QIR_TYPE_MAP.get(ftype)
        if qir_type is not None:
            value = self.format_value(qir_type, val)
            validated = self.validate_tag_and_value(tag, val)
            if validated:
                qo.write(f"OUTPUT\t{qir_type}\t{value}\t{tag}\n")

    def format_value(self, type_str: str, val):
        """Format the value if required"""
        if type_str != "BOOL":
            return val

        # For BOOLs, the L4 API will always return 0 or 1
        return "true" if val else "false"

    def write_shot(self, qo: StringIO, shot: QsysShot):
        """Format the user defined output from shots"""
        qo.write("START\n")
        self.emit_values_in_shot(qo, shot)
        self.shot_footer(qo)

    def write_first_shot(
        self, qo: StringIO, shot: QsysShot, attributes: Dict[str, str | None]
    ):
        """Write the first shot, which includes extra metadata"""
        self.first_shot_header(qo, attributes)
        self.emit_values_in_shot(qo, shot)
        self.shot_footer(qo)

    def emit_values_in_shot(self, qo: StringIO, shot: QsysShot):
        """Given a shot, check the format and emit each user value"""
        for cvar in shot:
            if cvar is not None and len(cvar) >= 2 and cvar[0] is not None:
                fields = cvar[0].split(":", maxsplit=2)
                if len(fields) >= 3 and fields[0] == "USER":
                    self.emit(qo, fields[1], fields[2], cvar[1])

    def qir_labeled_output(
        self, results: QsysShots, attributes: Dict[str, str | None]
    ) -> str:
        """
        Given a list of results associated with an `n_qubits` job, return
        the results in QIR "Labeled" Output Schema format.
        """
        if len(results) == 0:
            return ""

        qo = StringIO()
        self.results_header(qo)

        first_shot, results = results[0], results[1:]
        self.write_first_shot(qo, first_shot, attributes)
        for shot in results:
            self.write_shot(qo, shot)
        return qo.getvalue()
