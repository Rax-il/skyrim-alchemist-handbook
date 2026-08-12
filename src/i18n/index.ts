import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import ru from "./locales/ru/translation.json";
import en from "./locales/en/translation.json";
import fr from "./locales/fr/translation.json";
import de from "./locales/de/translation.json";
import it from "./locales/it/translation.json";
import es from "./locales/es/translation.json";
import pl from "./locales/pl/translation.json";
import ja from "./locales/ja/translation.json";
import zhHant from "./locales/zh-Hant/translation.json";

i18n.use(initReactI18next).init({
  resources: {
    ru: { translation: ru },
    en: { translation: en },
    fr: { translation: fr },
    de: { translation: de },
    it: { translation: it },
    es: { translation: es },
    pl: { translation: pl },
    ja: { translation: ja },
    "zh-Hant": { translation: zhHant },
  },
  lng: "ru",
  fallbackLng: "ru",
  interpolation: { escapeValue: false },
});

export default i18n;
