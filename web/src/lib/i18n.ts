/**
 * Translation lookup.
 *
 * Only one bundle is ever fetched: `loadLocale` dynamically imports the selected
 * language, which Vite emits as a separate chunk. Locale changes are rare and
 * only happen in the setup wizard, so a tiny store is enough to re-render.
 */
import { create } from 'zustand';

import type { Locale } from '@/lib/boot';
import { isLocale } from '@/lib/boot';
import en from '@/locales/en.json';

export type Dictionary = typeof en;

let messages: Dictionary = en;
let active: Locale = 'en';

/** Loads a bundle and switches the active dictionary. */
export async function loadLocale(locale: Locale): Promise<Dictionary> {
  if (locale === 'de') {
    const bundle = (await import('@/locales/de.json')).default as Dictionary;
    messages = bundle;
  } else {
    messages = en;
  }
  active = locale;
  return messages;
}

/** The currently active locale. */
export function currentLocale(): Locale {
  return active;
}

function resolve(path: string): string | undefined {
  let node: unknown = messages;
  for (const part of path.split('.')) {
    if (typeof node !== 'object' || node === null) return undefined;
    node = (node as Record<string, unknown>)[part];
  }
  return typeof node === 'string' ? node : undefined;
}

/**
 * Translates a dotted key and interpolates `{name}` placeholders.
 * Unknown keys return the key itself so missing strings are obvious in review.
 */
export function t(key: string, params?: Record<string, string | number>): string {
  const raw = resolve(key) ?? key;
  if (!params) return raw;
  return raw.replace(/\{(\w+)\}/g, (match, name: string) => {
    const value = params[name];
    return value === undefined ? match : String(value);
  });
}

/** Translates an API error key, which is always namespaced under `errors`. */
export function tError(key: string, params?: Record<string, string | number>): string {
  const translated = t(`errors.${key}`, params);
  return translated === `errors.${key}` ? t('errors.internal') : translated;
}

interface LocaleState {
  locale: Locale;
  setLocale: (locale: Locale) => Promise<void>;
}

/** Reactive locale for components that must re-render on a wizard language switch. */
export const useLocale = create<LocaleState>((set) => ({
  locale: active,
  setLocale: async (locale: Locale) => {
    if (!isLocale(locale)) return;
    await loadLocale(locale);
    active = locale;
    if (typeof document !== 'undefined') {
      document.documentElement.lang = locale;
    }
    set({ locale });
  },
}));
