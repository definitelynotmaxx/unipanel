import { useQuery } from '@tanstack/react-query';
import { healthQuery } from '@/lib/queries';
import { t } from '@/lib/i18n';

export function StatusPill() {
  const { isError } = useQuery(healthQuery);
  const isOk = !isError;

  const status = isOk ? 'online' : 'error';
  const label = status === 'online'
    ? t('shell.health.ok')
    : t('app.offline');

  return (
    <span className="chip">
      <span className={`dot dot-${status}`} />
      <span className="hidden sm:inline">{label}</span>
    </span>
  );
}
