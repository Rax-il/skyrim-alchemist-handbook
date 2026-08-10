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


# Known cases where multiple StringIDs share the exact same Russian text but
# resolve to different official English text — picked by hand after manual
# verification against the real game files (see final review, 2026-08-10).
# Keyed by ru_name -> (plugin, string_id).
#
# On the real 239-name dataset, 100 names have more than one exact-matching
# StringID across KNOWN_PLUGINS. For the vast majority of those, the
# candidate IDs presumably share the same effective English text, so the
# arbitrary "first match" pick below is harmless. These four are the only
# ones hand-confirmed to actually diverge in meaning; if a future name is
# found to diverge too, verify it against the real strings files and add it
# here rather than building general ambiguity-detection tooling.
OVERRIDES: dict[str, tuple[str, int]] = {
    # 24187 = "Slaughterfish Egg Nest" (a world-object nest), the wrong pick
    # by iteration order. 27849 = "Slaughterfish Egg", the alchemy ingredient.
    "Икра рыбы-убийцы": ("skyrim", 27849),
    # 26 candidate StringIDs share this ru text: 1 malformed (leading-space
    # " Magic Resistance"), 5 clean "Magic Resistance", 18 "Resist Magic".
    # 22468 is the lowest-numbered "Resist Magic" id (the majority/clean
    # text) and has full translations across all target languages.
    "Сопротивление магии": ("skyrim", 22468),
    # 13507 = "Magicka Damage" (wrong pick by iteration order). 52613 =
    # "Damage Magicka", matching the verb-first naming pattern already used
    # for the sibling property "Урон здоровью" -> "Damage Health".
    "Урон магии": ("skyrim", 52613),
    # 6510 = "Stamina Damage" (wrong pick by iteration order). 14785 =
    # "Damage Stamina", same verb-first pattern as above.
    "Урон запасу сил": ("skyrim", 14785),
}


def _plugin_path(strings_dir: str, plugin: str, lang: str) -> str:
    suffix = LANG_SUFFIX[lang]
    return os.path.join(strings_dir, f"{plugin}_{suffix}.strings")


# Cache of parsed ru plugin files, keyed by (ru_strings_dir, plugin), so that
# repeated resolve_id() calls against the same ru_strings_dir (e.g. once per
# name from translate_all) don't re-parse the same .strings files from disk
# every time. A missing file is cached as None (distinct from an empty-but-
# present file, which parses to {}) so its absence is remembered too. Keyed
# by the full ru_strings_dir path, so different directories never collide.
_ru_plugin_cache: dict[tuple[str, str], dict[int, str] | None] = {}


def _parse_ru_plugin(ru_strings_dir: str, plugin: str) -> dict[int, str] | None:
    key = (ru_strings_dir, plugin)
    if key not in _ru_plugin_cache:
        path = _plugin_path(ru_strings_dir, plugin, "ru")
        _ru_plugin_cache[key] = parse_strings(path) if os.path.exists(path) else None
    return _ru_plugin_cache[key]


def resolve_id(ru_name: str, ru_strings_dir: str) -> tuple[str, int] | None:
    if ru_name in OVERRIDES:
        return OVERRIDES[ru_name]

    parsed_by_plugin: dict[str, dict[int, str]] = {}
    for plugin in KNOWN_PLUGINS:
        strings = _parse_ru_plugin(ru_strings_dir, plugin)
        if strings is None:
            continue
        parsed_by_plugin[plugin] = strings

    exact_matches: list[tuple[str, int]] = []
    for plugin, strings in parsed_by_plugin.items():
        for sid, text in strings.items():
            if text == ru_name:
                exact_matches.append((plugin, sid))
    if exact_matches:
        return exact_matches[0]

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
                per_lang[lang] = text.strip()
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

    # Count actual translated values, not just matched ru_names: OVERRIDES
    # entries resolve their StringID without touching disk, so on a totally
    # bogus strings_root a handful of override names can still end up in
    # `translations` even though every per-lang lookup came back empty (the
    # lang .strings files don't exist at that path either). Checking
    # len(translations) alone would miss that and let a broken invocation
    # through with an empty-but-well-formed seed_translations.rs downstream.
    total_translated_values = sum(len(v) for v in translations.values())
    if total_translated_values == 0:
        print(
            "Ошибка: ничего не сопоставлено (0 переводов) — проверьте strings_root "
            f"({strings_root!r}): возможно, там нет каталога ru/strings/ или папок "
            "с нужными языками",
            file=sys.stderr,
        )
        sys.exit(1)

    print(json.dumps({"translations": translations, "unmatched": unmatched}, ensure_ascii=False, indent=2))
