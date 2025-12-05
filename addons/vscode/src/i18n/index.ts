import * as vscode from "vscode";
import {
  locales,
  supportedLanguages,
  LocaleKey,
  SupportedLanguage,
} from "./locales";

/**
 * Internationalization service for WhytCard
 * Uses centralized translations from locales.ts
 */
class I18n {
  private currentLanguage: SupportedLanguage = "en";

  setLanguage(lang: string): void {
    if (lang in locales) {
      this.currentLanguage = lang as SupportedLanguage;
    } else {
      this.currentLanguage = "en";
    }
  }

  getLanguage(): SupportedLanguage {
    return this.currentLanguage;
  }

  /**
   * Get translated string by key
   * @param key - The translation key
   * @param params - Optional parameters to interpolate (e.g., {count: 5})
   */
  t(key: LocaleKey | string, params?: Record<string, string | number>): string {
    const translations = locales[this.currentLanguage];
    const fallback = locales["en"];

    // Type-safe access with fallback
    let value: string;
    if (key in translations) {
      value = translations[key as LocaleKey];
    } else if (key in fallback) {
      value = fallback[key as LocaleKey];
    } else {
      value = key;
    }

    if (params) {
      for (const [paramKey, paramValue] of Object.entries(params)) {
        value = value.replace(`{${paramKey}}`, String(paramValue));
      }
    }

    return value;
  }

  /**
   * Get all translations for current language (useful for webview)
   */
  getAllTranslations(): Record<string, string> {
    return { ...locales[this.currentLanguage] };
  }

  getAvailableLanguages(): typeof supportedLanguages {
    return supportedLanguages;
  }
}

export const i18n = new I18n();

/**
 * Detect the best language based on VS Code settings and system language
 */
export function detectLanguage(): SupportedLanguage {
  const config = vscode.workspace.getConfiguration("whytcard");
  const configLang = config.get<string>("language");

  // User explicitly chose a language
  if (configLang && configLang !== "auto" && configLang in locales) {
    return configLang as SupportedLanguage;
  }

  // Auto-detect from VS Code's UI language
  const vscodeLang = vscode.env.language;
  const langCode = vscodeLang.split("-")[0].toLowerCase();

  if (langCode in locales) {
    return langCode as SupportedLanguage;
  }

  return "en";
}

// Re-export types and data for convenience
export { LocaleKey, SupportedLanguage, supportedLanguages } from "./locales";
