import { useQuery } from '@tanstack/react-query';

import { EmptyState } from '@/components/EmptyState';
import { t } from '@/lib/i18n';
import { bootstrapQuery, infoQuery } from '@/lib/queries';

export function AdminPage() {
  const bootstrap = useQuery(bootstrapQuery);
  const info = useQuery({ ...infoQuery, enabled: !!bootstrap.data?.authenticated });

  const instanceName = info.data?.instance_name ?? bootstrap.data?.instance_name ?? '';
  const database = info.data?.database;
  const schemaVersion = info.data?.schema_version;
  const locale = info.data?.locale ?? bootstrap.data?.locale ?? 'en';
  const version = bootstrap.data?.version;
  const startedAt = info.data?.started_at;

  return (
    <div className="space-y-6 animate-in">
      {/* Page header */}
      <div className="section-header">
        <div>
          <p className="text-[10px] font-semibold uppercase tracking-[0.16em] text-text-muted mb-1">
            {t('admin.eyebrow')}
          </p>
          <h1 className="heading-lg">{t('admin.title')}</h1>
        </div>
      </div>

      {/* Instance config */}
      <div className="card p-5">
        <h2 className="heading-sm mb-4">{t('admin.instance.title')}</h2>
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-x-8 gap-y-3">
          <ConfigRow label={t('admin.instance.name')} value={instanceName || '—'} />
          <ConfigRow label={t('admin.instance.locale')} value={locale} />
          <ConfigRow label={t('admin.instance.version')} value={version ?? '—'} />
          <ConfigRow label={t('admin.instance.schema')} value={schemaVersion != null ? `v${schemaVersion}` : '—'} />
          <ConfigRow label={t('admin.instance.database')} value={database ?? '—'} />
          <ConfigRow label={t('admin.instance.started')} value={startedAt ? new Date(startedAt).toLocaleString() : '—'} />
        </div>
      </div>

      {/* System status */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <div className="card p-5">
          <h2 className="heading-sm mb-4">Security</h2>
          <div className="space-y-3">
            <SecurityItem label="CSRF Protection" status="active" />
            <SecurityItem label="Session Security" status="active" />
            <SecurityItem label="Password Hashing" status="active" value="Argon2id" />
          </div>
        </div>

        <div className="card p-5">
          <h2 className="heading-sm mb-4">Connectors</h2>
          <EmptyState
            icon="Server"
            title="No connectors configured"
            description="Add connectors in the connectors tab to manage resources from Docker, Pterodactyl, Proxmox and more."
          />
        </div>
      </div>
    </div>
  );
}

function ConfigRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex flex-col py-1">
      <span className="text-[10px] font-semibold uppercase tracking-[0.12em] text-text-muted">
        {label}
      </span>
      <span className="text-[14px] text-text-primary font-mono">{value}</span>
    </div>
  );
}

function SecurityItem({ label, status, value }: { label: string; status: 'active' | 'inactive'; value?: string }) {
  return (
    <div className="flex items-center justify-between">
      <span className="text-[13px] text-text-secondary">{label}</span>
      <div className="flex items-center gap-2">
        {value && <span className="mono text-[12px] text-text-primary">{value}</span>}
        <span
          className={`badge ${status === 'active' ? 'badge-online' : 'badge-offline'}`}
        >
          {status === 'active' ? 'Active' : 'Inactive'}
        </span>
      </div>
    </div>
  );
}
