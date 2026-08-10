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
