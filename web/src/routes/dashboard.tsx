import { useNavigate } from '@tanstack/react-router';
import { useQuery } from '@tanstack/react-query';

import { t } from '@/lib/i18n';
import { bootstrapQuery } from '@/lib/queries';

export function DashboardPage() {
  const navigate = useNavigate();

  const bootstrap = useQuery(bootstrapQuery);

  const authenticated = bootstrap.data?.authenticated ?? false;
  const version = bootstrap.data?.version;
  const instanceName = bootstrap.data?.instance_name ?? '';

  const goTo = (path: string) => void navigate({ to: path });

  return (
    <div className="space-y-6 animate-in">
      {/* Stats row */}
      <div className="grid-stats">
        <div className="card p-4">
          <div className="flex items-center gap-3 mb-3">
            <div className="grid h-9 w-9 place-items-center rounded-lg bg-bg-tertiary">
              <span className="text-mint" style={{ fontFamily: 'var(--font-display)', fontSize: '16px' }}>
                {version?.charAt(0) ?? '—'}
              </span>
            </div>
            <span className="text-[11px] font-semibold uppercase tracking-[0.1em] text-text-muted">
              Version
            </span>
          </div>
          <span className="text-[22px] font-medium tracking-tight text-text-primary" style={{ fontFamily: 'var(--font-display)' }}>
            {version ?? '—'}
          </span>
        </div>

        <div className="card p-4">
          <div className="flex items-center gap-3 mb-3">
            <div className="grid h-9 w-9 place-items-center rounded-lg bg-bg-tertiary">
              <span className="text-info" style={{ fontFamily: 'var(--font-display)', fontSize: '16px' }}>
                {instanceName?.charAt(0)?.toUpperCase() ?? 'U'}
              </span>
            </div>
            <span className="text-[11px] font-semibold uppercase tracking-[0.1em] text-text-muted">
              Instance
            </span>
          </div>
          <span className="text-[14px] text-text-primary truncate" title={instanceName}>
            {instanceName || '—'}
          </span>
        </div>

        <div className="card p-4">
          <div className="flex items-center gap-3 mb-3">
            <div className="grid h-9 w-9 place-items-center rounded-lg bg-bg-tertiary">
              <span className="text-sodium" style={{ fontFamily: 'var(--font-display)', fontSize: '16px' }}>
                {authenticated ? '✓' : '○'}
              </span>
            </div>
            <span className="text-[11px] font-semibold uppercase tracking-[0.1em] text-text-muted">
              Status
            </span>
          </div>
          <span className={`text-[14px] font-medium ${authenticated ? 'text-mint' : 'text-warning'}`}>
            {authenticated ? 'Online' : 'Setup required'}
          </span>
        </div>
      </div>

      {/* Quick actions */}
      <div className="card p-5">
        <h2 className="heading-sm mb-4">Quick navigation</h2>
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3">
          <button className="btn justify-start" onClick={() => goTo('/resources')}>
            <span className="h-4 w-4 opacity-60" style={{ fontFamily: 'var(--font-display)' }}>⊞</span>
            <div className="flex-1 text-left">
              <span className="text-[13px] font-medium">{t('nav.resources')}</span>
              <span className="block text-[11px] text-text-muted">{t('resources.eyebrow')}</span>
            </div>
          </button>
          <button className="btn justify-start" onClick={() => goTo('/admin')}>
            <span className="h-4 w-4 opacity-60" style={{ fontFamily: 'var(--font-display)' }}>⚙</span>
            <div className="flex-1 text-left">
              <span className="text-[13px] font-medium">{t('nav.admin')}</span>
              <span className="block text-[11px] text-text-muted">{t('admin.eyebrow')}</span>
            </div>
          </button>
        </div>
      </div>
    </div>
  );
}
