import { describe, expect, it } from 'vitest';

import { t } from './i18n';

describe('i18n', () => {
  it('returns a readable string', () => {
    expect(typeof t('app.name')).toBe('string');
    expect(t('app.name')).toBe('UniPanel');
  });
});