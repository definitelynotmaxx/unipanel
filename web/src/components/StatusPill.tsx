import { useQuery } from '@tanstack/react-query';

import { healthQuery } from '@/lib/queries';
import { t } from '@/lib/i18n';

/** Live server state, polled every 30 seconds from `/api/v1/system/health`. */
export function StatusPill() {
  const health = useQuery(healthQuery);
  const state = health.isError ? 'down' : health.isSuccess ? 'ok' : 'checking';
  const label =
    state === 'ok'
      ? t('shell.health.ok')
      : state === 'down'
        ? t('shell.health.down')
        : t('shell.health.checking');

  return (
    <span className="chip" title={health.data ? `v${health.data.version}` : undefined}>
      <span
        className={
          state === 'ok' ? 'dot dot-ok' : state === 'down' ? 'dot dot-down' : 'dot dot-warn'
        }
      />
      <span className="hidden sm:inline">{label}</span>
      <span className="sm:hidden">{state === 'ok' ? 'OK' : state === 'down' ? 'ERR' : '···'}</span>
    </span>
  );
}
