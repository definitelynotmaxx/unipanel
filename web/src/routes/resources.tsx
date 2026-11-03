import { AuthLayout } from '@/components/AuthLayout';
import { Schematic } from '@/components/Schematic';
import { IconPlus } from '@/components/icons';
import { t } from '@/lib/i18n';

export function ResourcesPage() {  return (
    <AuthLayout
      eyebrow={t('resources.eyebrow')}
      headline={t('resources.headline')}
      lead={t('resources.lead')}
    >
      <div className="grid grid-cols-1 gap-6 lg:grid-cols-12">
        <div className="panel ticks p-6 lg:col-span-4">
          <div className="flex items-center justify-between mb-6">
            <h3 className="text-lg font-semibold text-paper">{t('resources.catalog.title')}</h3>
            <button className="btn btn-primary btn-sm">
              <IconPlus width={14} height={14} className="mr-2" />
              {t('resources.catalog.actions.add')}
            </button>
          </div>

          <div className="space-y-2">
            <div className="flex items-center justify-between p-3 rounded border border-line hover:bg-ink-1/30 cursor-pointer">
              <div className="flex items-center gap-3">
                <div className="grid h-10 w-10 place-items-center rounded border border-line bg-ink-1/50">

                </div>
                <div>
                  <p className="text-sm font-medium text-paper">Server #1</p>
                  <p className="mono text-xs text-muted">minecraft.example.com</p>
                </div>
              </div>
              <span className="mono text-xs text-green-500">online</span>
            </div>

            <div className="flex items-center justify-between p-3 rounded border border-line hover:bg-ink-1/30 cursor-pointer">
              <div className="flex items-center gap-3">
                <div className="grid h-10 w-10 place-items-center rounded border border-line bg-ink-1/50">

                </div>
                <div>
                  <p className="text-sm font-medium text-paper-dim">Server #2</p>
                  <p className="mono text-xs text-muted">backup.server.org</p>
                </div>
              </div>
              <span className="mono text-xs text-amber-500">degraded</span>
            </div>

            <div className="flex items-center justify-between p-3 rounded border border-line hover:bg-ink-1/30 cursor-pointer">
              <div className="flex items-center gap-3">
                <div className="grid h-10 w-10 place-items-center rounded border border-line bg-ink-1/50">

                </div>
                <div>
                  <p className="text-sm font-medium text-muted">Server #3</p>
                  <p className="mono text-xs text-muted">offline.server.net</p>
                </div>
              </div>
              <span className="mono text-xs text-muted">offline</span>
            </div>
          </div>

          <div className="mt-6 pt-6 border-t border-line">
            <button className="btn btn-ghost btn-block">

              {t('resources.catalog.actions.refreshAll')}
            </button>
          </div>
        </div>

        <div className="panel ticks p-6 lg:col-span-8">
          <div className="flex items-center justify-between mb-6">
            <h3 className="text-lg font-semibold text-paper">{t('resources.details.title')}</h3>
            <button className="btn btn-secondary btn-sm">

              {t('resources.details.actions.search')}
            </button>
          </div>

          <div className="space-y-4">
            <div className="panel ticks p-4 hover:bg-ink-1/20 transition-colors">
              <div className="flex items-start justify-between mb-3">
                <div>
                  <h4 className="text-base font-medium text-paper">Server #1 - Production Minecraft</h4>
                  <p className="mono text-sm text-muted mt-1">ID: server-001 | Type: game_server</p>
                </div>
                <span className="mono text-xs px-2 py-1 rounded border border-green-500/30 bg-green-500/10 text-green-500">ACTIVE</span>
              </div>

              <div className="grid grid-cols-3 gap-4 mt-4">
                <div>
                  <p className="mono text-xs text-muted">{t('resources.details.cpu')}</p>
                  <p className="mono text-sm text-paper">--%</p>
                </div>
                <div>
                  <p className="mono text-xs text-muted">{t('resources.details.memory')}</p>
                  <p className="mono text-sm text-paper">-- / -- GB</p>
                </div>
                <div>
                  <p className="mono text-xs text-muted">{t('resources.details.uptime')}</p>
                  <p className="mono text-sm text-paper">--:--:--</p>
                </div>
              </div>

              <div className="flex gap-2 mt-4">
                <button className="btn btn-primary btn-sm">{t('resources.details.actions.start')}</button>
                <button className="btn btn-secondary btn-sm">{t('resources.details.actions.stop')}</button>
                <button className="btn btn-ghost btn-sm">{t('resources.details.actions.logs')}</button>
                <button className="btn btn-ghost btn-sm text-error">{t('resources.details.actions.delete')}</button>
              </div>
            </div>

            <div className="panel ticks p-4 opacity-60">
              <div className="flex items-start justify-between mb-3">
                <div>
                  <h4 className="text-base font-medium text-muted">Server #2 - Backup System</h4>
                  <p className="mono text-sm text-muted mt-1">ID: server-002 | Type: generic</p>
                </div>
                <span className="mono text-xs px-2 py-1 rounded border border-amber-500/30 bg-amber-500/10 text-amber-500">DEGRADED</span>
              </div>

              <div className="grid grid-cols-3 gap-4 mt-4">
                <div>
                  <p className="mono text-xs text-muted">{t('resources.details.cpu')}</p>
                  <p className="mono text-sm text-muted">--%</p>
                </div>
                <div>
                  <p className="mono text-xs text-muted">{t('resources.details.memory')}</p>
                  <p className="mono text-sm text-muted">-- / -- GB</p>
                </div>
                <div>
                  <p className="mono text-xs text-muted">{t('resources.details.uptime')}</p>
                  <p className="mono text-sm text-muted">--:--:--</p>
                </div>
              </div>

              <div className="flex gap-2 mt-4">
                <button className="btn btn-primary btn-sm" disabled>{t('resources.details.actions.start')}</button>
                <button className="btn btn-secondary btn-sm" disabled>{t('resources.details.actions.stop')}</button>
                <button className="btn btn-ghost btn-sm" disabled>{t('resources.details.actions.logs')}</button>
              </div>
            </div>
          </div>
        </div>

        <div className="panel ticks p-6 lg:col-span-4">
          <Schematic />
        </div>
      </div>
    </AuthLayout>
  );
}