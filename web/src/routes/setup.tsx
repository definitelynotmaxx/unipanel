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
  const [username, setUsername] = useState('admin');
  const [password, setPassword] = useState('');
  const [displayName, setDisplayName] = useState('');
  const [instanceName, setInstanceName] = useState('');

  const setup = useMutation({
    mutationFn: (payload: {
      locale: string;
      username: string;
      password: string;
      display_name: string;
      instance_name?: null | string;
    }) => api.setup(payload as never),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: queryKeys.bootstrap });
      await navigate({ to: '/login', replace: true });
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
      headline={t('setup.headline')}
      lead={t('setup.lead')}
      footer={
        <span className="mono text-[8px] uppercase tracking-[0.2em] text-line-strong">
          {locale === 'en' ? 'EN' : 'DE'}
        </span>
      }
    >
      <div className="panel ticks p-7 sm:p-8">
        <AuthHeading
          eyebrow={t('setup.eyebrow')}
          title={t('setup.title')}
          lead={t('setup.lead')}
          aside={
            <span className="mono text-[8px] uppercase tracking-[0.18em] text-muted">01 / 01</span>
          }
        />

        <form onSubmit={onSubmit} noValidate>
          <div className="space-y-5">
            <Field label={t('setup.locale')} htmlFor="locale">
              <select id="locale" className="input" value={locale} onChange={(e) => setLocale(e.target.value)}>
                <option value="en">English</option>
                <option value="de">Deutsch</option>
              </select>
            </Field>

            <Field label={t('setup.username')} htmlFor="username">
              <input id="username" className="input" value={username}
                onChange={(e) => setUsername(e.target.value)} required autoFocus />
            </Field>

            <Field label={t('setup.password')} htmlFor="password">
              <input id="password" className="input" type="password" value={password}
                onChange={(e) => setPassword(e.target.value)} required />
            </Field>

            <Field label={t('setup.displayName')} htmlFor="display-name">
              <input id="display-name" className="input" value={displayName}
                onChange={(e) => setDisplayName(e.target.value)} />
            </Field>

            <Field label={t('setup.instanceName')} htmlFor="instance-name">
              <input id="instance-name" className="input" value={instanceName}
                onChange={(e) => setInstanceName(e.target.value)} />
            </Field>

            {setup.error && <Alert>{errorMessage(setup.error)}</Alert>}
          </div>

          <button
            type="submit"
            className="btn btn-primary btn-block mt-8"
            disabled={setup.isPending || username.length === 0 || password.length === 0}
          >
            {setup.isPending ? t('setup.submitting') : t('setup.submit')}
            <IconArrowRight width={16} height={16} className={setup.isPending ? 'opacity-40' : ''} />
          </button>
        </form>

        <div className="mt-8 flex items-center gap-3 border-t border-line pt-6">
          <IconShield width={13} height={13} className="text-muted" />
          <p className="mono text-[8.5px] uppercase tracking-[0.14em] text-muted">
            {t('setup.security.footer')}
          </p>
        </div>
      </div>
    </AuthLayout>
  );
}