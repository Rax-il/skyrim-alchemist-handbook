# tools/i18n/test_extract_ru_names.py
import os
import tempfile
import shutil
from extract_ru_names import extract_names


def _write(root, rel_path, content):
    path = os.path.join(root, rel_path)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)


def test_extract_names_from_fixture_repo():
    root = tempfile.mkdtemp()
    try:
        _write(
            root,
            "src-tauri/src/seed_data.rs",
            'pub const PROPERTIES: &[(&str, &str)] = &[\n'
            '    ("Бешенство", "Яд"),\n'
            '    ("Паралич", "Яд"),\n'
            '];\n'
            'pub const INGREDIENTS: &[(&str, [&str; 4])] = &[\n'
            '    ("Белянка", ["Бешенство", "Паралич", "Паралич", "Паралич"]),\n'
            '];\n'
            'pub const DAWNGUARD_INGREDIENTS: &[&str] = &["Жёлтый горноцвет"];\n'
            'pub const DRAGONBORN_INGREDIENTS: &[&str] = &[];\n'
            'pub const HEARTHFIRE_INGREDIENTS: &[&str] = &[];\n',
        )
        _write(
            root,
            "src-tauri/src/rare_curios.rs",
            'pub const RARE_CURIOS_PROPERTIES: &[(&str, &str)] = &[\n'
            '    ("Свет", "Улучшение"),\n'
            '];\n'
            'pub const RARE_CURIOS_INGREDIENTS: &[(&str, [&str; 4])] = &[\n'
            '    ("Банглер Бейн", ["Свет", "Свет", "Свет", "Свет"]),\n'
            '];\n',
        )
        _write(
            root,
            "src-tauri/src/creations.rs",
            'pub const CREATION_PROPERTIES: &[(&str, &str)] = &[\n'
            '    ("Повышение искусства убеждать", "Улучшение"),\n'
            '];\n'
            'pub const FISHING_INGREDIENTS: &[&str] = &["Стеклянный окунь"];\n'
            'pub const SAINTS_INGREDIENTS: &[&str] = &[];\n'
            'pub const PLAGUE_INGREDIENTS: &[&str] = &["Смертная плоть"];\n',
        )
        result = extract_names(root)
    finally:
        shutil.rmtree(root)

    assert result["properties"] == sorted(
        ["Бешенство", "Паралич", "Свет", "Повышение искусства убеждать"]
    )
    assert result["components"] == sorted(
        ["Белянка", "Жёлтый горноцвет", "Банглер Бейн", "Стеклянный окунь", "Смертная плоть"]
    )


if __name__ == "__main__":
    test_extract_names_from_fixture_repo()
    print("OK")
