import { AuthLayout } from '@/components/AuthLayout';
import { Schematic } from '@/components/Schematic';
import { IconPlus } from '@/components/icons';
import { t } from '@/lib/i18n';

export function AdminPage() {
  return (
    <AuthLayout
      eyebrow={t('admin.eyebrow')}
      headline={t('admin.headline')}
      lead={t('admin.lead')}
    >
      <div className="grid grid-cols-1 gap-6 lg:grid-cols-12">
        <div className="panel ticks p-6 lg:col-span-4">
          <h3 className="text-lg font-semibold text-paper mb-6">{t('admin.connectors.title')}</h3>

          <div className="space-y-3">
            <div className="panel ticks p-4 hover:bg-ink-1/20 transition-colors cursor-pointer">
              <div className="flex items-center gap-3 mb-3">
                <div className="grid h-10 w-10 place-items-center rounded border border-line bg-ink-1/50">
                  <IconPlus width={18} height={18} className="text-blue-500" />
                </div>
                <div className="flex-1">
                  <p className="text-sm font-medium text-paper">Minecraft Server</p>
                  <p className="mono text-xs text-muted">Game servers and world management</p>
                </div>
                <span className="mono text-xs px-2 py-1 rounded border border-green-500/30 bg-green-500/10 text-green-500">ACTIVE</span>
              </div>

              <div className="grid grid-cols-2 gap-2 mt-3">
                <div className="flex items-center gap-2">
                  <span className="mono text-xs text-muted">API configured</span>
                </div>
                <div className="flex items-center gap-2">
                  <span className="mono text-xs text-muted">BungeeCord</span>
                </div>
                <div className="flex items-center gap-2">
                  <span className="mono text-xs text-muted">World templates</span>
                </div>
                <div className="flex items-center gap-2">
                  <span className="mono text-xs text-muted">Auto-create</span>
                </div>
              </div>

              <button className="btn btn-primary btn-block mt-4">{t('admin.connectors.actions.manage')}</button>
            </div>

            <div className="panel ticks p-4 hover:bg-ink-1/20 transition-colors cursor-pointer opacity-70">
              <div className="flex items-center gap-3 mb-3">
                <div className="grid h-10 w-10 place-items-center rounded border border-line bg-ink-1/50">
                  <IconPlus width={18} height={18} className="text-muted" />
                </div>
                <div className="flex-1">
                  <p className="text-sm font-medium text-muted">Pterodactyl Panel</p>
                  <p className="mono text-xs text-muted">Server management via Pterodactyl API</p>
                </div>
                <span className="mono text-xs px-2 py-1 rounded border border-amber-500/30 bg-amber-500/10 text-amber-500">CONFIGURED</span>
              </div>

              <div className="grid grid-cols-2 gap-2 mt-3">
                <div className="flex items-center gap-2">
                  <span className="mono text-xs text-muted">API configured</span>
                </div>
                <div className="flex items-center gap-2">
                  <span className="mono text-xs text-muted">Multiple servers</span>
                </div>
              </div>

              <button className="btn btn-primary btn-block mt-4" disabled>{t('admin.connectors.actions.manage')}</button>
            </div>
          </div>

          <button className="btn btn-secondary btn-block mt-6">
            <IconPlus width={14} height={14} className="mr-2" />
            {t('admin.connectors.actions.addNew')}
          </button>
        </div>

        <div className="panel ticks p-6 lg:col-span-8">
          <h3 className="text-lg font-semibold text-paper mb-6">{t('admin.system.title')}</h3>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="panel ticks p-5">
              <h4 className="text-base font-medium text-paper mb-4">{t('admin.system.security.title')}</h4>
              <div className="space-y-3">
                <div className="flex items-center justify-between">
                  <span className="mono text-sm text-muted">{t('admin.system.security.csrfProtection')}</span>
                  <span className="mono text-xs px-2 py-1 rounded border border-green-500/30 bg-green-500/10 text-green-500">ENABLED</span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="mono text-sm text-muted">{t('admin.system.security.rateLimiting')}</span>
                  <span className="mono text-xs px-2 py-1 rounded border border-green-500/30 bg-green-500/10 text-green-500">ACTIVE</span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="mono text-sm text-muted">{t('admin.system.security.sessionSecurity')}</span>
                  <span className="mono text-xs px-2 py-1 rounded border border-green-500/30 bg-green-500/10 text-green-500">ENCRYPTED</span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="mono text-sm text-muted">{t('admin.system.security.masterKey')}</span>
                  <span className="mono text-xs px-2 py-1 rounded border border-green-500/30 bg-green-500/10 text-green-500">SECURE</span>
                </div>
              </div>
            </div>

            <div className="panel ticks p-5">
              <h4 className="text-base font-medium text-paper mb-4">{t('admin.system.performance.title')}</h4>
              <div className="space-y-3">
                <div className="flex items-center justify-between">
                  <span className="mono text-sm text-muted">{t('admin.system.performance.uptime')}</span>
                  <span className="mono text-sm text-paper">--:--:--</span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="mono text-sm text-muted">{t('admin.system.performance.memoryUsage')}</span>
                  <span className="mono text-sm text-paper">-- / -- MB</span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="mono text-sm text-muted">{t('admin.system.performance.cpuUsage')}</span>
                  <span className="mono text-sm text-paper">--%</span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="mono text-sm text-muted">{t('admin.system.performance.database')}</span>
                  <span className="mono text-xs px-2 py-1 rounded border border-green-500/30 bg-green-500/10 text-green-500">OPTIMIZED</span>
                </div>
              </div>
            </div>

            <div className="panel ticks p-5 md:col-span-2">
              <h4 className="text-base font-medium text-paper mb-4">{t('admin.system.configuration.title')}</h4>
              <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                <div>
                  <p className="mono text-xs text-muted mb-2">{t('admin.system.configuration.bindAddress')}</p>
                  <p className="mono text-sm text-paper">0.0.0.0:8080</p>
                </div>
                <div>
                  <p className="mono text-xs text-muted mb-2">{t('admin.system.configuration.dataDirectory')}</p>
                  <p className="mono text-sm text-paper">./data</p>
                </div>
                <div>
                  <p className="mono text-xs text-muted mb-2">{t('admin.system.configuration.database')}</p>
                  <p className="mono text-sm text-paper">SQLite (file)</p>
                </div>
                <div>
                  <p className="mono text-xs text-muted mb-2">{t('admin.system.configuration.masterKey')}</p>
                  <p className="mono text-sm text-amber-500">******</p>
                </div>
                <div>
                  <p className="mono text-xs text-muted mb-2">{t('admin.system.configuration.locale')}</p>
                  <p className="mono text-sm text-paper">EN/DE (configurable)</p>
                </div>
                <div>
                  <p className="mono text-xs text-muted mb-2">{t('admin.system.configuration.trustProxy')}</p>
                  <p className="mono text-sm text-paper">false</p>
                </div>
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