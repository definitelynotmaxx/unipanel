import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useNavigate } from '@tanstack/react-router';
import { useState, type FormEvent } from 'react';

import { AuthHeading, AuthLayout } from '@/components/AuthLayout';
import { Alert, Field } from '@/components/form';
import { IconArrowRight, IconShield } from '@/components/icons';
import { api, queryKeys } from '@/lib/api';
import { errorMessage } from '@/lib/errors';
import { t } from '@/lib/i18n';
import { bootstrapQuery } from '@/lib/queries';

export function SetupPage() {
  const bootstrap = useQuery(bootstrapQuery);
  const queryClient = useQueryClient();
  const navigate = useNavigate();

  const [instanceName, setInstanceName] = useState('');

  const setup = useMutation({
    mutationFn: api.setup,
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: queryKeys.bootstrap });
      await navigate({ to: '/login', replace: true });
    },
  });

  const onSubmit = (event: FormEvent) => {
    event.preventDefault();
    if (setup.isPending) return;
    setup.mutate({ instanceName: instanceName.trim() || 'UniPanel' });
  };

  const locale = bootstrap.data?.locale;

  return (
    <AuthLayout
      eyebrow={t('setup.eyebrow')}
      headline={
        <>
          {t('setup.headline.prefix')}
          <span className="text-sodium">{t('setup.headline.accent')}</span>
          {t('setup.headline.suffix')}
        </>
      }
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
            <Field
              label={t('setup.instanceName')}
              htmlFor="instance-name"
              hint={t('setup.instanceNameHint')}
            >
              <input
                id="instance-name"
                name="instance-name"
                className="input"
                value={instanceName}
                onChange={(event) => setInstanceName(event.target.value)}
                placeholder={t('setup.instanceNamePlaceholder')}
                autoComplete="off"
                required
                autoFocus
              />
            </Field>

            {setup.error && <Alert>{errorMessage(setup.error)}</Alert>}
          </div>

          <button
            type="submit"
            className="btn btn-primary btn-block mt-8"
            disabled={setup.isPending || instanceName.length === 0}
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