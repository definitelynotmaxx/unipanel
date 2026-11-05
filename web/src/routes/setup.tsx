import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useNavigate } from '@tanstack/react-router';
import { useState, type FormEvent } from 'react';

import { AuthHeading, AuthLayout } from '@/components/AuthLayout';
import { Alert, Field } from '@/components/form';
import { IconArrowRight, IconShield } from '@/components/icons';
import { api, queryKeys } from '@/lib/api';
import { errorMessage } from '@/lib/errors';
import { t } from '@/lib/i18n';

export function SetupPage() {
  const queryClient = useQueryClient();
  const navigate = useNavigate();

  const [locale, setLocale] = useState('en');
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [displayName, setDisplayName] = useState('');
  const [instanceName, setInstanceName] = useState('');

  const setup = useMutation({
    mutationFn: (payload: {
      locale: string;
      username: string;
      password: string;
      display_name: string;
      instance_name?: string | null;
    }) => api.setup(payload as never),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: queryKeys.bootstrap });
      await navigate({ to: '/', replace: true });
    },
  });

  const onSubmit = (event: FormEvent) => {
    event.preventDefault();
    if (setup.isPending) return;
    setup.mutate({
      locale,
      username: username.trim(),
      password,
      display_name: displayName.trim() || username.trim(),
      instance_name: instanceName.trim() || null,
    });
  };

  return (
    <AuthLayout
      eyebrow={t('setup.eyebrow')}
      headline={
        <>
          First run.<br />
          <span className="text-sodium">Set it up.</span>
        </>
      }
      lead={t('setup.account.lead')}
    >
      <div className="card p-6 sm:p-8">
        <AuthHeading
          eyebrow={t('setup.eyebrow')}
          title={t('setup.account.title')}
          lead={t('setup.account.lead')}
        />

        <form onSubmit={onSubmit} noValidate>
          <div className="space-y-4">
            <Field label={t('setup.locale')} htmlFor="locale">
              <select
                id="locale"
                className="input"
                value={locale}
                onChange={(e) => setLocale(e.target.value)}
              >
                <option value="en">English</option>
                <option value="de">Deutsch</option>
              </select>
            </Field>

            <Field label={t('setup.username')} htmlFor="setup-username">
              <input
                id="setup-username"
                className="input"
                value={username}
                onChange={(e) => setUsername(e.target.value)}
                placeholder="admin"
                required
                autoFocus
              />
            </Field>

            <Field label={t('setup.password')} htmlFor="setup-password">
              <input
                id="setup-password"
                className="input"
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                placeholder={t('setup.account.passwordHint')}
                required
              />
            </Field>

            <Field label={t('setup.displayName')} htmlFor="setup-display">
              <input
                id="setup-display"
                className="input"
                value={displayName}
                onChange={(e) => setDisplayName(e.target.value)}
                placeholder={t('setup.account.displayNamePlaceholder')}
              />
            </Field>

            <Field label={t('setup.instanceName')} htmlFor="setup-instance">
              <input
                id="setup-instance"
                className="input"
                value={instanceName}
                onChange={(e) => setInstanceName(e.target.value)}
                placeholder={t('setup.instanceNamePlaceholder')}
              />
            </Field>

            {setup.error && <Alert>{errorMessage(setup.error)}</Alert>}
          </div>

          <button
            type="submit"
            className="btn btn-primary btn-block mt-6"
            disabled={setup.isPending || !username || !password}
          >
            {setup.isPending ? t('setup.submitting') : t('setup.submit')}
            <IconArrowRight className="h-4 w-4 opacity-60" />
          </button>
        </form>

        <div className="mt-6 flex items-center gap-3 pt-4 border-t border-border">
          <IconShield className="h-3.5 w-3.5 text-text-muted shrink-0" />
          <p className="text-[10px] text-text-muted leading-relaxed">
            Your instance is secured with Argon2id password hashing.
          </p>
        </div>
      </div>
    </AuthLayout>
  );
}
