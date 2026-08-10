# Офлайн-пайплайн локализованных названий (План A) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** офлайн-инструмент (не часть рантайма приложения), который по уже
известным русским названиям 178 ингредиентов и 61+7 свойств находит их
`StringID` в официальных файлах локализации Skyrim SE и вытаскивает
официальные переводы на 8 остальных языков, готовые для вставки в БД.

**Architecture:** три независимых Python-скрипта в `tools/i18n/`, каждый со
своим чистым входом/выходом, соединённые в цепочку через промежуточные JSON-
файлы: `strings_format.py` (парсер бинарного формата `.strings`) →
`extract_ru_names.py` (текущие русские названия из наших `.rs`-файлов) →
`match_translations.py` (сопоставление по `StringID` + перевод) →
`generate_seed_translations.rs.py` (рендер результата в Rust-литералы).
Ничего из этого не трогает `src-tauri/src/{db,commands,lib}.rs` — выход
пайплайна (`seed_translations.rs`) в этом плане только генерируется внутри
`tools/i18n/` и **не** копируется в `src-tauri/src/` и не подключается к
сборке (перенос файла и подключение к сборке — это уже задача плана B).

**Tech Stack:** Python 3 (stdlib only — `struct`, `json`, `re`, `glob`, без
внешних зависимостей вроде pytest, тесты — обычные assert-скрипты).

## Global Constraints

- Ничего в `tools/i18n/` не должно требовать сети или системы сборки
  приложения — это разовый инструмент разработчика, запускается вручную.
- Все скрипты — чистые функции + тонкий `if __name__ == "__main__":` CLI
  поверх них, чтобы каждую функцию можно было протестировать без реальных
  файлов игры (см. секцию Testing в каждой задаче).
- Формат `.strings` подтверждён вручную на реальном файле
  `skyrim_chinese.strings` (see conversation): `uint32 count, uint32
  data_size`, затем `count × (uint32 StringID, uint32 offset)`, затем блок
  null-terminated UTF-8 строк; `offset` отсчитывается от начала блока строк
  (сразу после таблицы), а не от начала файла.
- `StringID` уникален только **внутри одного файла плагина**
  (`skyrim_russian.strings`, `dawnguard_russian.strings`, ...), не глобально
  — подтверждено на реальных данных (один и тот же ID в разных плагинах
  означает разные строки). Сопоставление всегда идёт в рамках одного и того
  же плагина у обоих языков.
- Официальный текст расходится с нашими данными по "ё" (игра пишет "е") и
  изредка по регистру — сопоставление должно сначала пробовать точное
  совпадение, и только если оно не нашлось — совпадение без учёта "ё" и
  регистра.

---

### Task 1: Парсер формата `.strings`

**Files:**
- Create: `tools/i18n/strings_format.py`
- Test: `tools/i18n/test_strings_format.py`

**Interfaces:**
- Produces: `parse_strings(path: str) -> dict[int, str]` — читает файл,
  возвращает `{StringID: текст}`.

- [ ] **Step 1: Написать проваливающийся тест**

```python
# tools/i18n/test_strings_format.py
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
```

- [ ] **Step 2: Убедиться, что тест падает**

Run: `cd tools/i18n && python3 test_strings_format.py`
Expected: `ModuleNotFoundError: No module named 'strings_format'`

- [ ] **Step 3: Написать реализацию**

```python
# tools/i18n/strings_format.py
"""Парсер бинарного формата Bethesda .strings (не .dlstrings/.ilstrings —
у тех есть дополнительный uint32-префикс длины перед каждой строкой,
здесь не нужен). Формат подтверждён вручную на реальном
skyrim_chinese.strings 2026-08-10."""
import struct


def parse_strings(path: str) -> dict[int, str]:
    with open(path, "rb") as f:
        data = f.read()

    count, _data_size = struct.unpack_from("<II", data, 0)
    table_offset = 8
    strings_start = table_offset + count * 8

    result: dict[int, str] = {}
    for i in range(count):
        string_id, rel_offset = struct.unpack_from("<II", data, table_offset + i * 8)
        start = strings_start + rel_offset
        end = data.find(b"\x00", start)
        result[string_id] = data[start:end].decode("utf-8", errors="replace")
    return result
```

- [ ] **Step 4: Убедиться, что тест проходит**

Run: `cd tools/i18n && python3 test_strings_format.py`
Expected: `OK`

- [ ] **Step 5: Прогнать парсер на реальном файле (ручная проверка, не автотест)**

Run:
```bash
python3 -c "
from strings_format import parse_strings
d = parse_strings('$HOME/Рабочий стол/alchemist-tauri-strings-src/ru/strings/skyrim_russian.strings')
print(len(d), 'строк')
print(d[1])
"
```
Expected: несколько десятков тысяч строк, без исключений.

---

### Task 2: Извлечение текущих русских названий из `.rs`-файлов

**Files:**
- Create: `tools/i18n/extract_ru_names.py`
- Test: `tools/i18n/test_extract_ru_names.py`

**Interfaces:**
- Consumes: ничего из Task 1.
- Produces: `extract_names(repo_root: str) -> dict[str, list[str]]` — словарь с
  двумя ключами: `"components"` и `"properties"`, каждый — отсортированный
  список уникальных русских названий (без привязки к дополнению — на Task 3
  сопоставление перебирает все плагины, а не полагается на текущую, местами
  ошибочную, разметку `addon`, см. Global Constraints выше).

- [ ] **Step 1: Написать проваливающийся тест**

```python
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
```

- [ ] **Step 2: Убедиться, что тест падает**

Run: `cd tools/i18n && python3 test_extract_ru_names.py`
Expected: `ModuleNotFoundError: No module named 'extract_ru_names'`

- [ ] **Step 3: Написать реализацию**

```python
# tools/i18n/extract_ru_names.py
"""Достаёт текущие русские названия ингредиентов/свойств прямо из
Rust-исходников (seed_data.rs/rare_curios.rs/creations.rs) через регэкспы,
завязанные на текущий стиль литералов в этих файлах. Хрупко к смене
форматирования — это разовый инструмент, не часть сборки, при поломке
достаточно поправить регэкспы под новый формат."""
import os
import re


def _read(repo_root: str, rel_path: str) -> str:
    with open(os.path.join(repo_root, "src-tauri/src", rel_path), encoding="utf-8") as f:
        return f.read()


def _prop_names(text: str, const_name: str) -> list[str]:
    m = re.search(rf'pub const {const_name}[^=]*=\s*&\[(.*?)\];', text, re.S)
    if not m:
        return []
    return re.findall(r'\("([^"]+)",\s*"(?:Улучшение|Яд)"\)', m.group(1))


def _ingredient_names_with_props(text: str, const_name: str) -> list[str]:
    m = re.search(rf'pub const {const_name}[^=]*=\s*&\[(.*?)\n\];', text, re.S)
    if not m:
        return []
    return re.findall(r'\("([^"]+)",\s*\[', m.group(1))


def _str_list(text: str, const_name: str) -> list[str]:
    m = re.search(rf'pub const {const_name}[^=]*=\s*&\[(.*?)\];', text, re.S)
    if not m:
        return []
    return re.findall(r'"([^"]+)"', m.group(1))


def extract_names(repo_root: str) -> dict[str, list[str]]:
    seed = _read(repo_root, "seed_data.rs")
    rare = _read(repo_root, "rare_curios.rs")
    creations = _read(repo_root, "creations.rs")

    properties = set(_prop_names(seed, "PROPERTIES"))
    properties |= set(_prop_names(rare, "RARE_CURIOS_PROPERTIES"))
    properties |= set(_prop_names(creations, "CREATION_PROPERTIES"))

    components = set(_ingredient_names_with_props(seed, "INGREDIENTS"))
    components |= set(_str_list(seed, "DAWNGUARD_INGREDIENTS"))
    components |= set(_str_list(seed, "DRAGONBORN_INGREDIENTS"))
    components |= set(_str_list(seed, "HEARTHFIRE_INGREDIENTS"))
    components |= set(_ingredient_names_with_props(rare, "RARE_CURIOS_INGREDIENTS"))
    components |= set(_str_list(creations, "FISHING_INGREDIENTS"))
    components |= set(_str_list(creations, "SAINTS_INGREDIENTS"))
    components |= set(_str_list(creations, "PLAGUE_INGREDIENTS"))

    return {
        "components": sorted(components),
        "properties": sorted(properties),
    }


if __name__ == "__main__":
    import json
    import sys

    repo_root = sys.argv[1] if len(sys.argv) > 1 else "../.."
    names = extract_names(repo_root)
    print(json.dumps(names, ensure_ascii=False, indent=2))
```

- [ ] **Step 4: Убедиться, что тест проходит**

Run: `cd tools/i18n && python3 test_extract_ru_names.py`
Expected: `OK`

- [ ] **Step 5: Прогнать на реальном репозитории и сохранить результат**

Run: `cd tools/i18n && python3 extract_ru_names.py ../.. > ru_names.json`
Expected: JSON с `"components"` (178 записей) и `"properties"` (61+
дополнительные свойства Rare Curios/CC — итого известно 65 распределены
между ~239 записями суммарно, точное число не фиксируем как assert, так как
это живые игровые данные).

---

### Task 3: Сопоставление по `StringID` и перевод

**Files:**
- Create: `tools/i18n/match_translations.py`
- Test: `tools/i18n/test_match_translations.py`

**Interfaces:**
- Consumes: `parse_strings` из Task 1 (`strings_format.py`).
- Produces:
  - `KNOWN_PLUGINS: list[str]` — фиксированный порядковый список базовых
    имён плагинов для перебора: `["skyrim", "dawnguard", "dragonborn",
    "hearthfires", "ccbgssse037-curios", "ccbgssse001-fish",
    "ccbgssse025-advdsgs"]`.
  - `normalize(text: str) -> str` — `text.lower()` с заменой "ё"/"Ё" на
    "е"/"Е" (уже выполнено `.lower()`, так что реально только замена "ё").
  - `resolve_id(ru_name: str, ru_dir: str) -> tuple[str, int] | None` —
    возвращает `(plugin_base_name, string_id)` первого найденного точного
    совпадения по всем `KNOWN_PLUGINS`, а если точного нет — первого
    совпадения после `normalize()`. `ru_dir` — путь к папке вида
    `.../ru/strings/`.
  - `translate_all(names: list[str], strings_root: str, langs: list[str]) ->
    tuple[dict[str, dict[str, str]], list[str]]` — для каждого имени из
    `names` ищет его в `{strings_root}/ru/strings/`, и если нашлось — для
    каждого языка из `langs` берёт текст по тому же `(plugin, id)` из
    `{strings_root}/{lang}/strings/`; возвращает `(translations, unmatched)`,
    где `translations = {ru_name: {lang: text}}` (язык отсутствует в
    подсловаре, если файл перевода для него не нашёлся или не содержит этот
    id), `unmatched` — список имён, для которых не нашлось совпадения даже в
    ru.

- [ ] **Step 1: Написать проваливающийся тест**

```python
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


if __name__ == "__main__":
    test_normalize_folds_yo_and_case()
    test_resolve_id_exact_match()
    test_resolve_id_normalized_fallback()
    test_resolve_id_not_found_returns_none()
    test_translate_all_end_to_end()
    print("OK")
```

- [ ] **Step 2: Убедиться, что тест падает**

Run: `cd tools/i18n && python3 test_match_translations.py`
Expected: `ModuleNotFoundError: No module named 'match_translations'`

- [ ] **Step 3: Написать реализацию**

```python
# tools/i18n/match_translations.py
"""Сопоставляет наши русские названия с официальными строками игры по
StringID и достаёт перевод на нужные языки. StringID уникален только внутри
одного файла плагина — поэтому перебор идёт по фиксированному списку
плагинов, а не по единому объединённому словарю (см. Global Constraints в
плане — объединение по всем плагинам сразу даёт ложные совпадения)."""
import os

from strings_format import parse_strings

KNOWN_PLUGINS = [
    "skyrim",
    "dawnguard",
    "dragonborn",
    "hearthfires",
    "ccbgssse037-curios",
    "ccbgssse001-fish",
    "ccbgssse025-advdsgs",
]

LANG_SUFFIX = {
    "ru": "russian",
    "en": "english",
    "fr": "french",
    "de": "german",
    "it": "italian",
    "es": "spanish",
    "pl": "polish",
    "ja": "japanese",
    "zh": "chinese",
}


def normalize(text: str) -> str:
    return text.lower().replace("ё", "е")


def _plugin_path(strings_dir: str, plugin: str, lang: str) -> str:
    suffix = LANG_SUFFIX[lang]
    return os.path.join(strings_dir, f"{plugin}_{suffix}.strings")


def resolve_id(ru_name: str, ru_strings_dir: str) -> tuple[str, int] | None:
    parsed_by_plugin: dict[str, dict[int, str]] = {}
    for plugin in KNOWN_PLUGINS:
        path = _plugin_path(ru_strings_dir, plugin, "ru")
        if not os.path.exists(path):
            continue
        parsed_by_plugin[plugin] = parse_strings(path)

    for plugin, strings in parsed_by_plugin.items():
        for sid, text in strings.items():
            if text == ru_name:
                return plugin, sid

    target = normalize(ru_name)
    for plugin, strings in parsed_by_plugin.items():
        for sid, text in strings.items():
            if normalize(text) == target:
                return plugin, sid

    return None


def translate_all(
    names: list[str], strings_root: str, langs: list[str]
) -> tuple[dict[str, dict[str, str]], list[str]]:
    ru_dir = os.path.join(strings_root, "ru", "strings")
    translations: dict[str, dict[str, str]] = {}
    unmatched: list[str] = []

    lang_plugin_cache: dict[tuple[str, str], dict[int, str]] = {}

    def get_lang_plugin(lang: str, plugin: str) -> dict[int, str]:
        key = (lang, plugin)
        if key not in lang_plugin_cache:
            path = _plugin_path(os.path.join(strings_root, lang, "strings"), plugin, lang)
            lang_plugin_cache[key] = parse_strings(path) if os.path.exists(path) else {}
        return lang_plugin_cache[key]

    for name in names:
        found = resolve_id(name, ru_dir)
        if found is None:
            unmatched.append(name)
            continue
        plugin, sid = found
        per_lang: dict[str, str] = {}
        for lang in langs:
            text = get_lang_plugin(lang, plugin).get(sid)
            if text is not None:
                per_lang[lang] = text
        translations[name] = per_lang

    return translations, unmatched


if __name__ == "__main__":
    import json
    import sys

    if len(sys.argv) != 4:
        print(
            "Usage: match_translations.py <ru_names.json> <strings_root> <langs_csv>",
            file=sys.stderr,
        )
        sys.exit(1)

    with open(sys.argv[1], encoding="utf-8") as f:
        ru_names = json.load(f)
    strings_root = sys.argv[2]
    langs = sys.argv[3].split(",")

    all_names = ru_names["components"] + ru_names["properties"]
    translations, unmatched = translate_all(all_names, strings_root, langs)

    print(f"Сопоставлено: {len(translations)}/{len(all_names)}", file=sys.stderr)
    if unmatched:
        print(f"Не найдено ({len(unmatched)}): {unmatched}", file=sys.stderr)

    print(json.dumps({"translations": translations, "unmatched": unmatched}, ensure_ascii=False, indent=2))
```

- [ ] **Step 4: Убедиться, что тест проходит**

Run: `cd tools/i18n && python3 test_match_translations.py`
Expected: `OK`

- [ ] **Step 5: Прогнать на реальных данных (ручная проверка)**

Run:
```bash
cd tools/i18n
python3 match_translations.py ru_names.json "$HOME/Рабочий стол/alchemist-tauri-strings-src" en,fr,de,it,es,pl,ja,zh > translations.json
```
Expected (уже проверено вручную в этой сессии на выборке): сопоставление
234-238 из 239 имён, `unmatched` содержит максимум `["Смертная плоть"]` —
если появятся другие расхождения, разобраться так же, как в этой сессии
(точечный поиск подстроки в нужном файле плагина), а не считать это багом
пайплайна по умолчанию.

---

### Task 4: Генерация `seed_translations.rs`

**Files:**
- Create: `tools/i18n/generate_seed_translations.py`
- Test: `tools/i18n/test_generate_seed_translations.py`

**Interfaces:**
- Consumes: результат `translate_all` из Task 3 — `dict[str, dict[str,
  str]]` (`{ru_name: {lang: text}}`).
- Produces: `render_rust(translations: dict[str, dict[str, str]]) -> str` —
  текст Rust-файла с двумя массивами троек `(ru_name, lang, text)`,
  отсортированными по `ru_name`, затем по `lang`, для стабильного diff между
  перегенерациями.

- [ ] **Step 1: Написать проваливающийся тест**

```python
# tools/i18n/test_generate_seed_translations.py
from generate_seed_translations import render_rust


def test_render_rust_basic_escaping_and_order():
    translations = {
        "Белянка": {"en": "White Cap", "de": 'Weißkappe "special"'},
        "Бешенство": {"en": "Frenzy"},
    }
    result = render_rust(translations)

    assert result.startswith("// Автосгенерировано tools/i18n/generate_seed_translations.py")
    assert 'pub const TRANSLATIONS: &[(&str, &str, &str)] = &[' in result
    assert '("Белянка", "de", "Weißkappe \\"special\\""),' in result
    assert '("Белянка", "en", "White Cap"),' in result
    assert '("Бешенство", "en", "Frenzy"),' in result
    # Белянка/de должна идти раньше Белянка/en (сортировка по языку внутри имени)
    assert result.index('"de"') < result.index('"en"')


if __name__ == "__main__":
    test_render_rust_basic_escaping_and_order()
    print("OK")
```

- [ ] **Step 2: Убедиться, что тест падает**

Run: `cd tools/i18n && python3 test_generate_seed_translations.py`
Expected: `ModuleNotFoundError: No module named 'generate_seed_translations'`

- [ ] **Step 3: Написать реализацию**

```python
# tools/i18n/generate_seed_translations.py
"""Рендерит результат match_translations.py в Rust-литералы того же стиля,
что и seed_data.rs. Выходной файл — сгенерированные данные, вручную не
редактируется (перегенерируется этим скриптом при обновлении переводов)."""


def _escape(text: str) -> str:
    return text.replace("\\", "\\\\").replace('"', '\\"')


def render_rust(translations: dict[str, dict[str, str]]) -> str:
    rows: list[tuple[str, str, str]] = []
    for ru_name in sorted(translations):
        for lang in sorted(translations[ru_name]):
            rows.append((ru_name, lang, translations[ru_name][lang]))

    lines = [
        "// Автосгенерировано tools/i18n/generate_seed_translations.py —",
        "// не редактировать вручную, перегенерировать скриптом при обновлении.",
        "pub const TRANSLATIONS: &[(&str, &str, &str)] = &[",
    ]
    for ru_name, lang, text in rows:
        lines.append(f'    ("{_escape(ru_name)}", "{lang}", "{_escape(text)}"),')
    lines.append("];")
    lines.append("")
    return "\n".join(lines)


if __name__ == "__main__":
    import json
    import sys

    if len(sys.argv) != 2:
        print("Usage: generate_seed_translations.py <translations.json>", file=sys.stderr)
        sys.exit(1)

    with open(sys.argv[1], encoding="utf-8") as f:
        payload = json.load(f)

    print(render_rust(payload["translations"]))
```

- [ ] **Step 4: Убедиться, что тест проходит**

Run: `cd tools/i18n && python3 test_generate_seed_translations.py`
Expected: `OK`

- [ ] **Step 5: Сгенерировать реальный файл (ручной шаг)**

Run:
```bash
cd tools/i18n
python3 generate_seed_translations.py translations.json > seed_translations.rs
wc -l seed_translations.rs
```
Ожидается порядка 1600-1900 строк (239 имён × до 8 языков). Файл пока **не**
копируется в `src-tauri/src/` и не подключается к сборке — это задача плана
B (там же и понадобится решить, что делать с оставшимся `unmatched` списком,
это уже вопрос UX редактирования, отложенный на отдельное обсуждение).

---

## Итоговая проверка плана

- [ ] Все четыре скрипта проходят свои тесты: `for f in test_*.py; do python3
  "$f" || echo "FAIL: $f"; done` внутри `tools/i18n/` — ничего не должно
  напечатать `FAIL`.
- [ ] Ручной прогон Task 3/Task 5 на реальных данных (`ru_names.json` +
  скачанные архивы) даёт коэффициент совпадения не ниже уже подтверждённых
  97%, а список `unmatched` — короткий и объяснимый (как "Смертная плоть"
  выше), а не длинный и необъяснённый.
