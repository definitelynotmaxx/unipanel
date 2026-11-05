import { useId, useState, type InputHTMLAttributes, type ReactNode } from 'react';
import { AlertCircle, Eye, EyeOff } from 'lucide-react';

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

export function Field({ label, htmlFor, hint, error, optional, children }: FieldProps) {
  return (
    <div className="form-group">
      <label htmlFor={htmlFor} className="form-label">
        {label}
        {optional && <span className="ml-1 text-[10px] font-normal text-text-muted">{t('app.optional')}</span>}
      </label>
      {children}
      {error ? (
        <p className="form-error" role="alert">{error}</p>
      ) : hint ? (
        <p className="form-hint">{hint}</p>
      ) : null}
    </div>
  );
}

interface PasswordInputProps extends Omit<InputHTMLAttributes<HTMLInputElement>, 'type'> {
  invalid?: boolean;
}

export function PasswordInput({ invalid, ...props }: PasswordInputProps) {
  const [visible, setVisible] = useState(false);
  return (
    <div className="relative">
      <input
        {...props}
        type={visible ? 'text' : 'password'}
        className="input pr-10"
        aria-invalid={invalid ? 'true' : undefined}
      />
      <button
        type="button"
        className="absolute right-0 top-0 h-full w-10 grid place-items-center text-text-muted hover:text-text-primary transition-colors"
        onClick={() => setVisible((v) => !v)}
        aria-pressed={visible}
        aria-label={t(visible ? 'login.hidePassword' : 'login.showPassword')}
      >
        {visible
          ? <EyeOff className="h-4 w-4" />
          : <Eye className="h-4 w-4" />}
      </button>
    </div>
  );
}

export function Alert({ children }: { children: ReactNode }) {
  return (
    <div className="alert alert-danger" role="alert">
      <AlertCircle className="h-4 w-4 shrink-0 mt-0.5" />
      <span>{children}</span>
    </div>
  );
}

export function WarnAlert({ children }: { children: ReactNode }) {
  return (
    <div className="alert alert-warn" role="note">
      <span className="h-1.5 w-1.5 shrink-0 rounded-full bg-amber mt-1 block" aria-hidden="true" />
      <span>{children}</span>
    </div>
  );
}

export function StrengthMeter({ strength }: { strength: Strength }) {
  const id = useId();
  const colors = ['bg-text-muted', 'bg-danger', 'bg-amber', 'bg-mint', 'bg-mint'];
  return (
    <div className="mt-2">
      <div className="flex items-baseline justify-between">
        <span className="text-[9px] font-semibold uppercase tracking-[0.14em] text-text-muted">
          {t('setup.account.strength.label')}
        </span>
        <span
          className={`mono text-[10px] uppercase tracking-[0.14em] ${
            strength.level === 0 ? 'text-text-muted' :
            strength.level === 1 ? 'text-danger' :
            strength.level === 2 ? 'text-amber' :
            'text-mint'
          }`}
          aria-live="polite"
          id={id}
        >
          {t(strength.labelKey)}
        </span>
      </div>
      <div className="mt-1.5 flex gap-1" role="progressbar" aria-valuenow={strength.level} aria-labelledby={id}>
        {[0, 1, 2, 3].map((i) => (
          <div
            key={i}
            className={`h-1 flex-1 rounded-full transition-colors ${
              i < strength.level ? colors[strength.level] : 'bg-bg-tertiary'
            }`}
          />
        ))}
      </div>
    </div>
  );
}
