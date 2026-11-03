import { useId, useState, type InputHTMLAttributes, type ReactNode } from 'react';

import { IconAlert, IconEye, IconEyeOff } from '@/components/icons';
import { cx } from '@/lib/cx';
import { t } from '@/lib/i18n';
import type { Strength } from '@/lib/strength';

interface FieldProps {
  label: string;
  htmlFor: string;
  hint?: string;
  error?: string;
  optional?: boolean;
  children: ReactNode;
}

/** Label, control and hint with the panel's mono caption style. */
export function Field({ label, htmlFor, hint, error, optional, children }: FieldProps) {
  return (
    <div className="field">
      <div className="field-label">
        <label htmlFor={htmlFor}>{label}</label>
        {optional && <span className="text-[9px] text-muted">{t('app.optional')}</span>}
      </div>
      {children}
      {error ? (
        <p className="field-hint text-danger" role="alert">
          {error}
        </p>
      ) : hint ? (
        <p className="field-hint">{hint}</p>
      ) : null}
    </div>
  );
}

interface PasswordInputProps extends Omit<InputHTMLAttributes<HTMLInputElement>, 'type'> {
  invalid?: boolean;
}

/** Password input with a show/hide toggle that keeps the caret position. */
export function PasswordInput({ invalid, ...props }: PasswordInputProps) {
  const [visible, setVisible] = useState(false);
  return (
    <div className="input-with-action">
      <input
        {...props}
        type={visible ? 'text' : 'password'}
        className="input"
        aria-invalid={invalid ? 'true' : undefined}
      />
      <button
        type="button"
        className="input-action"
        onClick={() => setVisible((value) => !value)}
        aria-pressed={visible}
        aria-label={t(visible ? 'login.hidePassword' : 'login.showPassword')}
      >
        {visible ? <IconEyeOff width={15} height={15} /> : <IconEye width={15} height={15} />}
      </button>
    </div>
  );
}

/** Inline error banner used by the auth screens. */
export function Alert({ children }: { children: ReactNode }) {
  return (
    <div className="alert" role="alert">
      <IconAlert width={15} height={15} className="mt-0.5 shrink-0" />
      <span>{children}</span>
    </div>
  );
}

/** Warning banner, used for the permanent language choice. */
export function WarnAlert({ children }: { children: ReactNode }) {
  return (
    <div className="alert alert-warn" role="note">
      <span className="mt-1 block h-1.5 w-1.5 shrink-0 bg-amber" aria-hidden="true" />
      <span>{children}</span>
    </div>
  );
}

/** Four-segment strength meter driven by `lib/strength`. */
export function StrengthMeter({ strength }: { strength: Strength }) {
  const id = useId();
  return (
    <div className="mt-2">
      <div className="flex items-baseline justify-between">
        <span className="eyebrow text-[8px]">{t('setup.account.strength.label')}</span>
        <span
          className={cx(
            'mono text-[10px] uppercase tracking-[0.14em]',
            strength.level === 0 && 'text-muted',
            strength.level === 1 && 'text-danger',
            strength.level === 2 && 'text-amber',
            strength.level >= 3 && 'text-mint',
          )}
          aria-live="polite"
          id={id}
        >
          {t(strength.labelKey)}
        </span>
      </div>
      <div className="meter mt-1.5" role="progressbar" aria-valuemin={0} aria-valuemax={4} aria-valuenow={strength.level} aria-labelledby={id}>
        {[0, 1, 2, 3].map((index) => (
          <span
            key={index}
            className="meter__bar"
            data-level={index < strength.level ? strength.level : 0}
          />
        ))}
      </div>
    </div>
  );
}
