"""Test the QIR results formatter"""

import typing
from io import StringIO
from typing import Dict, Optional, Union

import pytest

from qir_formatter.labeled_formatter import QirLabeledFormatter, QsysShots

formatting_test_data = [
    ("INT", "i0", 42, "OUTPUT\tINT\t42\ti0\n"),
    ("UINT", "ui0", 99, "OUTPUT\tINT\t99\tui0\n"),
    ("FLOAT", "f0", 3.1415926, "OUTPUT\tDOUBLE\t3.1415926\tf0\n"),
    ("BOOL", "b0", 0, "OUTPUT\tBOOL\tfalse\tb0\n"),
    ("RESULT", "r0", 0, "OUTPUT\tRESULT\t0\tr0\n"),
    ("QIRARRAY", "a0", 4, "OUTPUT\tARRAY\t4\ta0\n"),
    ("QIRTUPLE", "t0", 13, "OUTPUT\tTUPLE\t13\tt0\n"),
]


@pytest.mark.parametrize("ftype,tag,value,expected", formatting_test_data)
def test_formatting(ftype: str, tag: str, value: int, expected: str) -> None:
    """Test raw data types that are rendered to QIR output."""
    qo = StringIO()
    QirLabeledFormatter().emit(qo, ftype, tag, value)
    assert qo.getvalue() == expected
    qo.seek(0)
    qo.truncate(0)


@pytest.mark.parametrize("ftype", ["BOOLARR", "INTARR", "UINTARR", "FLOATARR"])
def test_ignored_types(ftype: str) -> None:
    """Test valid raw data types are ignored."""
    qo = StringIO()
    QirLabeledFormatter().emit(qo, ftype, "aggr", [1, 1, 0])
    assert qo.getvalue() == ""
    qo.seek(0)
    qo.truncate(0)


def test_undefined_type() -> None:
    "Test invalid raw data type."
    qo = StringIO()
    QirLabeledFormatter().emit(qo, "UINT32", "aggr", [1, 1, 0])
    assert qo.getvalue() == ""


malformed_testdata = [
    ("INT", 0, 0),
    ("INT", "syndrome0", None),
    ("BOOL", 5.4321, None),
    ("FLOAT", None, 99),
    (None, None, None),
]


@pytest.mark.parametrize("ftype,tag,value", malformed_testdata)
def test_malformed(
    ftype: Optional[str],
    tag: Optional[Union[str, float]],
    value: Optional[int],
) -> None:
    "Test improperly formatted raw data."
    qo = StringIO()
    QirLabeledFormatter().emit(qo, ftype, tag, value)  # type: ignore
    assert qo.getvalue() == ""
    qo.seek(0)
    qo.truncate(0)


def test_result_list() -> None:
    """Test complete formatting of a valid list of raw results."""

    # 4 shots of a variety of results
    results: QsysShots = [
        [("USER:INT:i1", 42), ("USER:BOOL:b1", 1)],
        [
            ("USER:RESULT:r1", 99),
            ("USER:INT:i2", -54321),
            ("USER:FLOAT:f1", 2.71828182845904523536),
        ],
        [
            ("USER:FLOAT:φ", 3.14159265358979323848),
            ("USER:INT:large", pow(2, 63) - 1),
            ("USER:INT:neg", -pow(2, 63)),
        ],
        [
            ("USER:QIRARRAY:0_a", 2),
            ("USER:QIRTUPLE:1_a0t", 2),
            ("USER:INT:2_a0t0i", 42),
            ("USER:RESULT:3_a0t1r", 0),
            ("USER:QIRTUPLE:4_a1t", 2),
            ("USER:INT:5_a1t0i", 33),
            ("USER:RESULT:6_a1t1r", 1),
        ],
    ]

    attributes: Dict[str, str | None] = {
        "qir_profiles": "base_profile",
        "required_num_qubits": "9",
        "required_num_results": "9",
    }
    qir_output = QirLabeledFormatter().qir_labeled_output(results, attributes)
    with open("src/tests/data/good1.output", encoding="utf-8") as f:
        assert f.read() == qir_output


qir_attributes = {
    "qir_profiles": "base_profile",
    "required_num_qubits": "9",
    "required_num_results": "3",
}
testdata = [(qir_attributes, "full1.output"), ({}, "missing_attributes.output")]


@pytest.mark.parametrize("attributes,output_file", testdata)
def test_full_raw_result_list(
    attributes: Dict[str, str | None],
    output_file: str,
) -> None:
    """
    Test complete formatting of a valid list of raw results, with
    some fields that are not rendered for QIR output.
    """
    results: QsysShots = [
        [
            ("MEAS:BOOL:", 1),
            ("MEAS:BOOL:", 1),
            ("MEAS:BOOL:", 0),
            ("USER:INT:reg1", 3),
            ("USER:FLOAT:reg2", 1.43),
            ("USER:INT:reg:3", 1),
            ("METRICS:INT:SQ", 12),
            ("METRICS:INT:TQ", 15),
            ("METRICS:INT:SPAM", 8),
            ("METRICS:INT:NumQubits", 4),
            ("METRICS:FLOAT:ShotTime", 0.0194),
        ],
        [
            ("MEAS:BOOL:", 0),
            ("MEAS:BOOL:", 1),
            ("MEAS:BOOL:", 0),
            ("MEAS:BOOL:", 1),
            ("USER:INT:reg1", 9),
            ("MEAS:BOOL:", 1),
            ("MEAS:BOOL:", 0),
            ("USER:INTARR:reg3", [5, 12]),
            ("METRICS:INT:SQ", 15),
            ("METRICS:INT:TQ", 19),
            ("METRICS:INT:SPAM", 12),
            ("METRICS:FLOAT:ShotTime", 0.0394),
        ],
        [
            ("MEAS:BOOL:", 1),
            ("MEAS:BOOL:", 1),
            ("MEAS:BOOL:", 1),
            ("MEAS:BOOL:", 1),
            ("USER:INT:reg1", -2),
            ("EXIT:INT:Unexpected results indicating deep flaw/exit", 1000),
            ("METRICS:INT:SQ", 12),
            ("METRICS:INT:TQ", 15),
            ("METRICS:INT:SPAM", 8),
            ("METRICS:FLOAT:ShotTime", 0.0194),
        ],
    ]

    qir_output = QirLabeledFormatter().qir_labeled_output(results, attributes)
    with open(f"src/tests/data/{output_file}", encoding="utf-8") as f:
        assert f.read() == qir_output


# Test case with intentionally malformed data doesn't need to be type checked
@typing.no_type_check
def test_malformed_raw_result_list() -> None:
    """
    Test complete formatting of a valid list of raw results, with
    some malformed entries.
    """
    results = [
        [
            ("USER:INT:reg1", 3),
            ("USER::reg3", 1.414),
            ("USER:RESULT:r1", None),
            ("USER:FLOAT:reg2", 1.43),
            (":INT:i0", 1),
            ("::i0", 1),
            (":::", 1),
            ("::", 0),
            (":", 0),
            (":::"),
            ("::::::"),
            ("foo"),
            ("foo::bar"),
            [],
            [[]],
            None,
            [None, -42],
            ("USER:BOOL:", 1),
        ],
    ]

    attributes = {
        "qir_profiles": "base_profile",
        "required_num_qubits": 9,
        "required_num_results": 9,
    }
    qir_output = QirLabeledFormatter().qir_labeled_output(results, attributes)
    with open("src/tests/data/malformed1.output", encoding="utf-8") as f:
        assert f.read() == qir_output


def test_empty_tag_submission() -> None:
    """Test empty tag submission produces output."""
    results: QsysShots = [
        [
            ("USER:QIRARRAY:", 2),
            ("USER:QIRTUPLE:", 2),
        ]
    ]
    attributes: Dict[str, str | None] = {}
    qir_output = QirLabeledFormatter().qir_labeled_output(results, attributes)
    # assert that the output still shows with ARRAY and TUPLE types
    assert "OUTPUT\tARRAY\t2\t\n" in qir_output
    assert "OUTPUT\tTUPLE\t2\t\n" in qir_output
