# tools/i18n/test_match_translations.py
import os
import struct
import tempfile
import shutil
from match_translations import normalize, resolve_id, translate_all


def _write_strings(path, entries):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    data = b""
    offsets = []
    for _sid, text in entries:
        offsets.append(len(data))
        data += text.encode("utf-8") + b"\x00"
    header = struct.pack("<II", len(entries), len(data))
    table = b"".join(
        struct.pack("<II", sid, off) for (sid, _t), off in zip(entries, offsets)
    )
    with open(path, "wb") as f:
        f.write(header + table + data)


def test_normalize_folds_yo_and_case():
    assert normalize("Жёлтый") == normalize("желтый")


def test_resolve_id_exact_match():
    root = tempfile.mkdtemp()
    try:
        _write_strings(
            os.path.join(root, "ru", "strings", "skyrim_russian.strings"),
            [(10, "Белянка")],
        )
        result = resolve_id("Белянка", os.path.join(root, "ru", "strings"))
        assert result == ("skyrim", 10)
    finally:
        shutil.rmtree(root)


def test_resolve_id_normalized_fallback():
    root = tempfile.mkdtemp()
    try:
        _write_strings(
            os.path.join(root, "ru", "strings", "dawnguard_russian.strings"),
            [(20, "Желтый горноцвет")],  # игра пишет без "ё"
        )
        result = resolve_id("Жёлтый горноцвет", os.path.join(root, "ru", "strings"))
        assert result == ("dawnguard", 20)
    finally:
        shutil.rmtree(root)


def test_resolve_id_not_found_returns_none():
    root = tempfile.mkdtemp()
    try:
        os.makedirs(os.path.join(root, "ru", "strings"))
        result = resolve_id("Нет такого", os.path.join(root, "ru", "strings"))
        assert result is None
    finally:
        shutil.rmtree(root)


def test_translate_all_end_to_end():
    root = tempfile.mkdtemp()
    try:
        _write_strings(
            os.path.join(root, "ru", "strings", "skyrim_russian.strings"),
            [(1, "Белянка"), (2, "Бешенство")],
        )
        _write_strings(
            os.path.join(root, "en", "strings", "skyrim_english.strings"),
            [(1, "White Cap"), (2, "Frenzy")],
        )
        # Немецкий файл вообще не подложен — должен просто не попасть в
        # результат для этого имени, без падения.
        translations, unmatched = translate_all(
            ["Белянка", "Бешенство", "Незнакомое"], root, ["en", "de"]
        )
    finally:
        shutil.rmtree(root)

    assert translations["Белянка"] == {"en": "White Cap"}
    assert translations["Бешенство"] == {"en": "Frenzy"}
    assert unmatched == ["Незнакомое"]


def test_resolve_id_and_translate_all_handle_same_id_in_different_plugins():
    # StringID=5 means something DIFFERENT in each plugin's file. A naive
    # implementation that merged all plugins' string tables into one dict
    # before matching (instead of keeping them per-plugin) would silently
    # return the wrong plugin's text for one of these two names. This is
    # exactly the failure mode the per-plugin, not-merged design exists to
    # prevent (see the module docstring / brief's Global Constraints).
    root = tempfile.mkdtemp()
    try:
        _write_strings(
            os.path.join(root, "ru", "strings", "skyrim_russian.strings"),
            [(5, "Роза")],
        )
        _write_strings(
            os.path.join(root, "ru", "strings", "dawnguard_russian.strings"),
            [(5, "Полынь")],
        )
        _write_strings(
            os.path.join(root, "en", "strings", "skyrim_english.strings"),
            [(5, "Rose")],
        )
        _write_strings(
            os.path.join(root, "en", "strings", "dawnguard_english.strings"),
            [(5, "Wormwood")],
        )

        ru_dir = os.path.join(root, "ru", "strings")
        assert resolve_id("Роза", ru_dir) == ("skyrim", 5)
        assert resolve_id("Полынь", ru_dir) == ("dawnguard", 5)

        translations, unmatched = translate_all(
            ["Роза", "Полынь"], root, ["en"]
        )
    finally:
        shutil.rmtree(root)

    assert unmatched == []
    assert translations["Роза"] == {"en": "Rose"}
    assert translations["Полынь"] == {"en": "Wormwood"}


if __name__ == "__main__":
    test_normalize_folds_yo_and_case()
    test_resolve_id_exact_match()
    test_resolve_id_normalized_fallback()
    test_resolve_id_not_found_returns_none()
    test_translate_all_end_to_end()
    test_resolve_id_and_translate_all_handle_same_id_in_different_plugins()
    print("OK")
