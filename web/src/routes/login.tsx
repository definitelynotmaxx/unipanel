import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useNavigate } from '@tanstack/react-router';
import { useState, type FormEvent } from 'react';

import { AuthHeading, AuthLayout } from '@/components/AuthLayout';
import { Alert, Field, PasswordInput } from '@/components/form';
import { IconArrowRight } from '@/components/icons';
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
      headline="Sign in"
      lead="Authenticate to reach the control plane."
    >
      <div className="card p-6 sm:p-8 shadow-sm">
        <AuthHeading
          eyebrow={instanceName}
          title="Sign in"
          lead="Enter your credentials to continue."
        />

        <form onSubmit={onSubmit} noValidate>
          <div className="space-y-4">
            <Field label={t('login.username')} htmlFor="username">
              <input
                id="username"
                className="input"
                value={username}
                onChange={(e) => setUsername(e.target.value)}
                placeholder="admin"
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
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                placeholder="Enter your password"
                autoComplete="current-password"
                required
                invalid={!!login.error}
              />
            </Field>

            {login.error && <Alert>{errorMessage(login.error)}</Alert>}
          </div>

          <button
            type="submit"
            className="btn btn-primary btn-block mt-6"
            disabled={login.isPending || !username || !password}
          >
            {login.isPending ? t('login.submitting') : t('login.submit')}
            <IconArrowRight className="h-4 w-4 opacity-60" />
          </button>
        </form>

        <p className="mt-4 text-center text-[10px] text-gray-400">
          Sessions expire after 30 days of inactivity.
        </p>
      </div>
    </AuthLayout>
  );
}
