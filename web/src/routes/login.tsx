import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useNavigate } from '@tanstack/react-router';
import { useState, type FormEvent } from 'react';

import { AuthHeading, AuthLayout } from '@/components/AuthLayout';
import { Alert, Field, PasswordInput } from '@/components/form';
import { IconArrowRight, IconShield } from '@/components/icons';
import { api, queryKeys } from '@/lib/api';
import { errorMessage } from '@/lib/errors';
import { t } from '@/lib/i18n';
import { bootstrapQuery } from '@/lib/queries';

export function LoginPage() {
  const bootstrap = useQuery(bootstrapQuery);
  const queryClient = useQueryClient();
  const navigate = useNavigate();

  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [capsLock, setCapsLock] = useState(false);

  const login = useMutation({
    mutationFn: api.login,
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: queryKeys.bootstrap });
      await navigate({ to: '/' });
    },
  });

  const onSubmit = (event: FormEvent) => {
    event.preventDefault();
    if (login.isPending) return;
    login.mutate({ username, password });
  };

  const instanceName = bootstrap.data?.instance_name ?? 'UniPanel';

  return (
    <AuthLayout
      eyebrow={t('login.eyebrow')}
      headline={
        <>
          One console.
          <br />
          <span className="text-sodium">Every</span> machine.
        </>
      }
      lead={t('login.lead')}
      footer={
        <span className="mono text-[8px] uppercase tracking-[0.2em] text-line-strong">
          LOCAL ACCOUNT
        </span>
      }
    >
      <div className="panel ticks p-7 sm:p-8">
        <AuthHeading
          eyebrow={instanceName}
          title={t('login.title')}
          lead={t('login.lead')}
          aside={
            <span className="mono text-[8px] uppercase tracking-[0.18em] text-muted">01 / 01</span>
          }
        />

        <form onSubmit={onSubmit} noValidate>
          <div className="space-y-4">
            <Field label={t('login.username')} htmlFor="username">
              <input
                id="username"
                name="username"
                className="input"
                value={username}
                onChange={(event) => setUsername(event.target.value)}
                placeholder={t('login.usernamePlaceholder')}
                autoComplete="username"
                autoCapitalize="none"
                autoCorrect="off"
                spellCheck={false}
                required
                autoFocus
              />
            </Field>

            <Field label={t('login.password')} htmlFor="password">
              <PasswordInput
                id="password"
                name="password"
                value={password}
                onChange={(event) => setPassword(event.target.value)}
                onKeyUp={(event) => setCapsLock(event.getModifierState?.('CapsLock') ?? false)}
                onKeyDown={(event) => setCapsLock(event.getModifierState?.('CapsLock') ?? false)}
                placeholder={t('login.passwordPlaceholder')}
                autoComplete="current-password"
                required
                invalid={Boolean(login.error)}
              />
            </Field>

            {capsLock && (
              <p className="mono text-[9px] uppercase tracking-[0.16em] text-amber">
                {t('login.capsLock')}
              </p>
            )}

            {login.error && <Alert>{errorMessage(login.error)}</Alert>}
          </div>

          <button
            type="submit"
            className="btn btn-primary btn-block mt-6"
            disabled={login.isPending || username.length === 0 || password.length === 0}
          >
            {login.isPending ? t('login.submitting') : t('login.submit')}
            <IconArrowRight width={16} height={16} className={login.isPending ? 'opacity-40' : ''} />
          </button>
        </form>

        <div className="mt-6 flex items-center gap-3 border-t border-line pt-4">
          <IconShield width={13} height={13} className="text-muted" />
          <p className="mono text-[8.5px] uppercase tracking-[0.14em] text-muted">
            {t('login.footer')}
          </p>
        </div>
      </div>
    </AuthLayout>
  );
}
