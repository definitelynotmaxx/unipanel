/** Turns an API error into a translated message. */
import { ApiError } from '@/lib/api';
import { t, tError } from '@/lib/i18n';

export function errorMessage(error: unknown): string {
  if (error instanceof ApiError) {
    const params: Record<string, string | number> = {};
    for (const [key, value] of Object.entries(error.params)) {
      if (typeof value === 'string' || typeof value === 'number') params[key] = value;
    }
    return tError(error.key, params);
  }
  if (error instanceof Error && error.message) return error.message;
  return t('errors.internal');
}

/** The message key of an API error, or `internal`. */
export function errorKey(error: unknown): string {
  return error instanceof ApiError ? error.key : 'internal';
}
