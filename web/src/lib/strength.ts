/**
 * Password strength, mirroring the backend policy in spirit.
 *
 * The server is authoritative; this only gives the operator a live hint while
 * typing. Length dominates, because that is what the policy rewards.
 */

export const MIN_PASSWORD_LENGTH = 10;

export type StrengthLevel = 0 | 1 | 2 | 3 | 4;

export interface Strength {
  level: StrengthLevel;
  /** Translation key for the label, e.g. `setup.account.strength.good`. */
  labelKey: string;
  /** Whether the password satisfies the known server rules. */
  acceptable: boolean;
}

const LABELS = [
  'setup.account.strength.empty',
  'setup.account.strength.weak',
  'setup.account.strength.fair',
  'setup.account.strength.good',
  'setup.account.strength.strong',
] as const;

const COMMON = new Set([
  '1234567890',
  'password123',
  'password1234',
  'qwertzuiop',
  'qwertyuiop',
  'iloveyou123',
  'adminadmin',
  'administrator',
  'letmein1234',
  'welcome12345',
  'changeme123',
  'unipanel123',
]);

/** Scores a password from 0 (empty) to 4 (strong). */
export function passwordStrength(password: string, accountNames: string[] = []): Strength {
  if (password.length === 0) {
    return { level: 0, labelKey: LABELS[0], acceptable: false };
  }

  const lowered = password.toLowerCase();
  const violatesName = accountNames.some((name) => {
    const trimmed = name.trim().toLowerCase();
    return trimmed.length >= 3 && lowered.includes(trimmed);
  });
  const common = COMMON.has(lowered);

  let score = 0;
  if (password.length >= MIN_PASSWORD_LENGTH) score += 1;
  if (password.length >= 16) score += 1;
  if (/[a-z]/.test(password) && /[A-Z]/.test(password)) score += 1;
  if (/\d/.test(password) && /[^A-Za-z0-9]/.test(password)) score += 1;
  if (new Set(password).size <= 4) score = Math.min(score, 1);

  if (common || violatesName) score = Math.min(score, 1);
  const level = Math.max(0, Math.min(4, score)) as StrengthLevel;

  return {
    level,
    labelKey: LABELS[level],
    acceptable:
      password.length >= MIN_PASSWORD_LENGTH &&
      password.length <= 200 &&
      !common &&
      !violatesName,
  };
}
