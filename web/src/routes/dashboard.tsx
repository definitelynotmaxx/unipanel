import { useQuery } from '@tanstack/react-query';
import { useNavigate } from '@tanstack/react-router';

import { AuthLayout } from '@/components/AuthLayout';
import { Schematic } from '@/components/Schematic';
import { IconSliders, IconStack } from '@/components/icons';
import { t } from '@/lib/i18n';
import { bootstrapQuery } from '@/lib/queries';

export function DashboardPage() {
  const bootstrap = useQuery(bootstrapQuery);
  const navigate = useNavigate();

  const version = bootstrap.data?.version;

  const navigateTo = (path: string) => {
    void navigate({ to: path });
  };

  return (
    <AuthLayout
      eyebrow={t('dashboard.eyebrow')}
      headline={t('dashboard.headline')}
      lead={t('dashboard.lead')}
    >
      <div className="grid grid-cols-1 gap-6 md:grid-cols-2 xl:grid-cols-3">
        <div className="panel ticks p-6 transition-all hover:scale-[1.02]">
          <div className="flex items-start justify-between mb-4">
            <div className="flex items-center gap-3">
              <div className="grid h-12 w-12 place-items-center rounded border border-line bg-ink-1/50">
                <IconStack width={20} height={20} className="text-sodium" />
              </div>
              <div>
                <h3 className="text-lg font-semibold text-paper">{t('dashboard.resources.title')}</h3>
                <p className="mono text-sm text-muted">{t('dashboard.resources.subtitle')}</p>
              </div>
            </div>
            <span className="mono text-xs text-muted">{t('dashboard.resources.status')}</span>
          </div>

          <div className="mt-6 space-y-3">
            <div className="flex items-baseline justify-between">
              <span className="mono text-sm text-muted">{t('dashboard.resources.total')}</span>
              <span className="mono text-2xl font-mono text-paper">00</span>
            </div>
            <div className="flex items-baseline justify-between">
              <span className="mono text-sm text-muted">{t('dashboard.resources.active')}</span>
              <span className="mono text-xl text-green-500">00</span>
            </div>
            <div className="flex items-baseline justify-between">
              <span className="mono text-sm text-muted">{t('dashboard.resources.offline')}</span>
              <span className="mono text-xl text-muted">00</span>
            </div>
          </div>

          <button
            onClick={() => navigateTo('/resources')}
            className="btn btn-primary btn-block mt-6"
          >
            {t('dashboard.resources.actions.manage')}
          </button>
        </div>

        <div className="panel ticks p-6 transition-all hover:scale-[1.02]">
          <div className="flex items-start justify-between mb-4">
            <div className="flex items-center gap-3">
              <div className="grid h-12 w-12 place-items-center rounded border border-line bg-ink-1/50">
                <IconSliders width={20} height={20} className="text-blue-500" />
              </div>
              <div>
                <h3 className="text-lg font-semibold text-paper">{t('dashboard.admin.title')}</h3>
                <p className="mono text-sm text-muted">{t('dashboard.admin.subtitle')}</p>
              </div>
            </div>
            <span className="mono text-xs text-muted">{t('dashboard.admin.status')}</span>
          </div>

          <div className="mt-6 space-y-3">
            <div className="flex items-baseline justify-between">
              <span className="mono text-sm text-muted">{t('dashboard.admin.connectors')}</span>
              <span className="mono text-2xl font-mono text-paper">00</span>
            </div>
            <div className="flex items-baseline justify-between">
              <span className="mono text-sm text-muted">{t('dashboard.admin.instances')}</span>
              <span className="mono text-xl text-paper">00</span>
            </div>
          </div>

          <button
            onClick={() => navigateTo('/admin')}
            className="btn btn-secondary btn-block mt-6"
          >
            {t('dashboard.admin.actions.configure')}
          </button>
        </div>

        <div className="panel ticks p-6 transition-all hover:scale-[1.02] md:col-span-2 xl:col-span-1">
          <div className="flex items-start justify-between mb-4">
            <div className="flex items-center gap-3">
              <div className="grid h-12 w-12 place-items-center rounded border border-line bg-ink-1/50">

              </div>
              <div>
                <h3 className="text-lg font-semibold text-paper">{t('dashboard.system.title')}</h3>
                <p className="mono text-sm text-muted">{t('dashboard.system.subtitle')}</p>
              </div>
            </div>
          </div>

          <div className="mt-6 space-y-4">
            <div className="flex items-center justify-between">
              <span className="mono text-sm text-muted">{t('dashboard.system.uptime')}</span>
              <span className="mono text-sm text-paper">--:--:--</span>
            </div>
            <div className="flex items-center justify-between">
              <span className="mono text-sm text-muted">{t('dashboard.system.version')}</span>
              <span className="mono text-sm text-paper">{version ?? '0.0.0'}</span>
            </div>
            <div className="flex items-center justify-between">
              <span className="mono text-sm text-muted">{t('dashboard.system.language')}</span>
              <span className="mono text-sm text-amber-500">EN/DE</span>
            </div>
          </div>

          <button
            onClick={() => navigateTo('/resources')}
            className="btn btn-ghost btn-block mt-6"
          >
            {t('dashboard.system.actions.viewLogs')}
          </button>
        </div>

        <div className="panel ticks p-6 md:col-span-2 xl:col-span-2">
          <h3 className="text-lg font-semibold text-paper mb-4">{t('dashboard.recent.title')}</h3>
          <div className="space-y-3">
            <div className="flex items-center justify-between py-3 border-b border-line/50">
              <div className="flex items-center gap-3">
                <div className="w-2 h-2 rounded-full bg-green-500" />
                <span className="mono text-sm text-paper">Server #1</span>
              </div>
              <span className="mono text-xs text-muted">online</span>
            </div>
            <div className="flex items-center justify-between py-3 border-b border-line/50">
              <div className="flex items-center gap-3">
                <div className="w-2 h-2 rounded-full bg-green-500" />
                <span className="mono text-sm text-muted">Server #2</span>
              </div>
              <span className="mono text-xs text-muted">online</span>
            </div>
            <div className="flex items-center justify-between py-3 border-b border-line/50">
              <div className="flex items-center gap-3">
                <div className="w-2 h-2 rounded-full bg-amber-500" />
                <span className="mono text-sm text-muted">Server #3</span>
              </div>
              <span className="mono text-xs text-muted">degraded</span>
            </div>
          </div>
        </div>

        <div className="panel ticks p-6 xl:col-span-1">
          <Schematic />
        </div>
      </div>
    </AuthLayout>
  );
}