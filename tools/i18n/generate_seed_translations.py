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
