// addons.rs — идентификатор источника ингредиента (официальное дополнение,
// сторонний мод/творение или добавлено вручную через "Редактировать базу").
// Единственный источник истины для допустимых значений колонки
// components.addon и для валидации сохранённого набора включённых
// дополнений в settings.json (см. layout.rs). Строковое представление
// (snake_case через serde) — стабильный идентификатор, не привязанный к
// отображаемому названию, чтобы смена языка интерфейса в будущем не
// затрагивала ни базу, ни settings.json (см. фронтенд lib/addons.ts —
// там же лежат отображаемые названия).
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Addon {
    BaseGame,
    Dawnguard,
    Dragonborn,
    Hearthfire,
    RareCurios,
    Fishing,
    SaintsAndSeducers,
    PlagueOfTheDead,
    /// Добавлено пользователем через "Редактировать базу" (insert_component).
    /// На момент введения этого поля таких компонентов в базе ещё нет ни
    /// одного — появятся только через явное добавление в редакторе.
    UserAdded,
}

impl Addon {
    pub const ALL: [Addon; 9] = [
        Addon::BaseGame,
        Addon::Dawnguard,
        Addon::Dragonborn,
        Addon::Hearthfire,
        Addon::RareCurios,
        Addon::Fishing,
        Addon::SaintsAndSeducers,
        Addon::PlagueOfTheDead,
        Addon::UserAdded,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Addon::BaseGame => "base_game",
            Addon::Dawnguard => "dawnguard",
            Addon::Dragonborn => "dragonborn",
            Addon::Hearthfire => "hearthfire",
            Addon::RareCurios => "rare_curios",
            Addon::Fishing => "fishing",
            Addon::SaintsAndSeducers => "saints_and_seducers",
            Addon::PlagueOfTheDead => "plague_of_the_dead",
            Addon::UserAdded => "user_added",
        }
    }

    /// Разбор из сохранённого settings.json — там дополнения хранятся как
    /// сырые строки (Vec<String>, а не Vec<Addon>) специально для того,
    /// чтобы одно неизвестное/устаревшее значение (ручная правка файла,
    /// более старая версия программы) не роняло разбор ВСЕГО settings.json
    /// целиком, а просто отбрасывалось — см. layout::load(). На границе же
    /// Tauri-команд (данные от фронтенда) используется строгий serde-разбор
    /// прямо в Vec<Addon> — там некорректное значение действительно баг.
    pub fn from_id(s: &str) -> Option<Addon> {
        Addon::ALL.into_iter().find(|a| a.as_str() == s)
    }
}
