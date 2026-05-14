"""Public package surface tests."""

from io import StringIO

from qir_formatter import QirLabeledFormatter, QsysShots


def test_top_level_exports_support_basic_usage() -> None:
    """The package should expose a stable top-level import surface."""
    results: QsysShots = [[("USER:INT:answer", 42)]]

    output = QirLabeledFormatter().qir_labeled_output(results, {})

    assert "OUTPUT\tINT\t42\tanswer\n" in output


def test_top_level_formatter_can_emit_values() -> None:
    """The main formatter should be directly importable from the package root."""
    out = StringIO()

    QirLabeledFormatter().emit(out, "RESULT_ARRAY", "bits", [1, 0, 1])

    assert out.getvalue() == "OUTPUT\tRESULT_ARRAY\t101\tbits\n"
