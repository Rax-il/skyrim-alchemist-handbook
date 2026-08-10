import struct
import tempfile
import os
from strings_format import parse_strings


def _build_fixture(entries):
    """entries: список (string_id, text). Строит байты в формате .strings."""
    data = b""
    offsets = []
    for _sid, text in entries:
        offsets.append(len(data))
        data += text.encode("utf-8") + b"\x00"
    header = struct.pack("<II", len(entries), len(data))
    table = b"".join(
        struct.pack("<II", sid, off) for (sid, _text), off in zip(entries, offsets)
    )
    return header + table + data


def test_parse_strings_basic():
    fixture = _build_fixture([(1, "Hello"), (2, "Мир"), (42, "")])
    fd, path = tempfile.mkstemp(suffix=".strings")
    os.close(fd)
    try:
        with open(path, "wb") as f:
            f.write(fixture)
        result = parse_strings(path)
    finally:
        os.remove(path)
    assert result == {1: "Hello", 2: "Мир", 42: ""}


def test_parse_strings_empty_file():
    fixture = _build_fixture([])
    fd, path = tempfile.mkstemp(suffix=".strings")
    os.close(fd)
    try:
        with open(path, "wb") as f:
            f.write(fixture)
        result = parse_strings(path)
    finally:
        os.remove(path)
    assert result == {}


if __name__ == "__main__":
    test_parse_strings_basic()
    test_parse_strings_empty_file()
    print("OK")
