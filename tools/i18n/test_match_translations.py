# tools/i18n/test_match_translations.py
import os
import struct
import tempfile
import shutil
from match_translations import OVERRIDES, normalize, resolve_id, translate_all


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


def test_resolve_id_uses_override_instead_of_first_exact_match():
    # Two plugins each have an exact-match candidate for the same ru_name.
    # Without an override, resolve_id would return the first one found
    # (skyrim, per KNOWN_PLUGINS order) — exactly the bug the final review
    # found for 4 real names (e.g. "Икра рыбы-убийцы" wrongly resolving to
    # the "Slaughterfish Egg Nest" world-object instead of the alchemy
    # ingredient "Slaughterfish Egg"). Register a fixture name in OVERRIDES
    # pointing at the *second* plugin's id and confirm it wins.
    root = tempfile.mkdtemp()
    name = "Тестовое Слово Для Оверрайда"
    try:
        _write_strings(
            os.path.join(root, "ru", "strings", "skyrim_russian.strings"),
            [(100, name)],
        )
        _write_strings(
            os.path.join(root, "ru", "strings", "dawnguard_russian.strings"),
            [(200, name)],
        )

        # Sanity check: without the override, the first plugin (skyrim, per
        # KNOWN_PLUGINS order) wins.
        assert resolve_id(name, os.path.join(root, "ru", "strings")) == ("skyrim", 100)

        OVERRIDES[name] = ("dawnguard", 200)
        try:
            result = resolve_id(name, os.path.join(root, "ru", "strings"))
        finally:
            del OVERRIDES[name]

        assert result == ("dawnguard", 200)
    finally:
        shutil.rmtree(root)


def test_translate_all_strips_whitespace_from_translated_text():
    # Real data has rows with leading/trailing spaces in the source .strings
    # file (e.g. a Polish "Odporność na magię " with a trailing space, or a
    # malformed " Magic Resistance" with a leading space). translate_all()
    # must not leak that whitespace into the stored translation.
    root = tempfile.mkdtemp()
    try:
        _write_strings(
            os.path.join(root, "ru", "strings", "skyrim_russian.strings"),
            [(1, "Белянка")],
        )
        _write_strings(
            os.path.join(root, "en", "strings", "skyrim_english.strings"),
            [(1, "  White Cap ")],
        )
        translations, unmatched = translate_all(["Белянка"], root, ["en"])
    finally:
        shutil.rmtree(root)

    assert translations["Белянка"] == {"en": "White Cap"}
    assert unmatched == []


def test_translate_all_override_name_against_missing_strings_root_yields_no_values():
    # Regression test for an interaction discovered while fixing the CLI's
    # "exit non-zero if nothing matched" check: an OVERRIDES name resolves
    # its (plugin, sid) without touching disk at all, so even against a
    # strings_root with NO ru/strings/ directory, an override name still
    # "matches" and ends up as a key in `translations` — just with an empty
    # per-lang dict, since the lang .strings files don't exist either. A
    # naive `len(translations) == 0` check in the CLI would miss this and
    # let a fully broken strings_root through. This test documents why the
    # CLI instead sums up actual translated *values* across all entries.
    root = tempfile.mkdtemp()
    name = "Тестовое Оверрайд Имя Без Файлов"
    try:
        shutil.rmtree(root)  # root itself doesn't exist -> no ru/strings/ either
        OVERRIDES[name] = ("skyrim", 999)
        try:
            translations, unmatched = translate_all([name], root, ["en"])
        finally:
            del OVERRIDES[name]
    finally:
        if os.path.exists(root):
            shutil.rmtree(root)

    assert unmatched == []
    assert translations == {name: {}}
    assert sum(len(v) for v in translations.values()) == 0


if __name__ == "__main__":
    test_normalize_folds_yo_and_case()
    test_resolve_id_exact_match()
    test_resolve_id_normalized_fallback()
    test_resolve_id_not_found_returns_none()
    test_translate_all_end_to_end()
    test_resolve_id_and_translate_all_handle_same_id_in_different_plugins()
    test_resolve_id_uses_override_instead_of_first_exact_match()
    test_translate_all_strips_whitespace_from_translated_text()
    test_translate_all_override_name_against_missing_strings_root_yields_no_values()
    print("OK")
