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
