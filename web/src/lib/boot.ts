/**
 * Values the server injects into `index.html` before the bundle runs.
 *
 * The locale is fixed per instance, so it is read once here and used to load
 * exactly one translation bundle.
 */
export interface Boot {
  locale: string;
  setupRequired: boolean;
  instanceName: string;
  version: string;
}

declare global {
  interface Window {
    __UNIPANEL__?: Partial<Boot>;
  }
}

const fallback: Boot = {
  locale: 'en',
  setupRequired: false,
  instanceName: 'UniPanel',
  version: '0.0.0',
};

let cached: Boot | null = null;

/** Reads the injected boot payload, tolerating a missing script (dev server). */
export function readBoot(): Boot {
  if (cached) return cached;
  const injected = typeof window === 'undefined' ? undefined : window.__UNIPANEL__;
  cached = {
    locale: typeof injected?.locale === 'string' ? injected.locale : fallback.locale,
    setupRequired: Boolean(injected?.setupRequired),
    instanceName:
      typeof injected?.instanceName === 'string' && injected.instanceName.trim()
        ? injected.instanceName
        : fallback.instanceName,
    version: typeof injected?.version === 'string' ? injected.version : fallback.version,
  };
  return cached;
}

/** Supported locales. Kept in sync with `unipanel_core::Locale`. */
export type Locale = 'en' | 'de';

export function isLocale(value: string): value is Locale {
  return value === 'en' || value === 'de';
}

/** The instance locale, falling back to English for unknown values. */
export function bootLocale(): Locale {
  const { locale } = readBoot();
  return isLocale(locale) ? locale : 'en';
}
