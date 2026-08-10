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
