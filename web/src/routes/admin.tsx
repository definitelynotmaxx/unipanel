import { useQuery } from '@tanstack/react-query';
import { bootstrapQuery, infoQuery } from '@/lib/queries';

export function AdminPage() {
  const bootstrap = useQuery(bootstrapQuery);
  const info = useQuery({ ...infoQuery, enabled: !!bootstrap.data?.authenticated });

  const version = bootstrap.data?.version ?? '—';
  const instanceName = bootstrap.data?.instance_name ?? 'UniPanel';
  const database = info.data?.database ?? '—';
  const schemaVersion = info.data?.schema_version;
  const startedAt = info.data?.started_at;
  const locale = info.data?.locale ?? bootstrap.data?.locale ?? 'en';

  return (
    <div className="space-y-5 animate-in">
      {/* ── Title ── */}
      <div>
        <h1 className="heading-lg">Administration</h1>
        <p className="text-sm text-gray-500 mt-1">
          Instance settings, security, and connectors
        </p>
      </div>

      {/* ── Instance info ── */}
      <div className="card p-6">
        <h2 className="text-sm font-semibold text-gray-800 mb-4">Instance</h2>
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-x-8 gap-y-4">
          <InfoRow label="Name" value={instanceName} />
          <InfoRow label="Language" value={locale} />
          <InfoRow label="Version" value={version} />
          <InfoRow label="Schema" value={schemaVersion != null ? `v${schemaVersion}` : '—'} />
          <InfoRow label="Database" value={database} />
          <InfoRow label="Started" value={startedAt ? new Date(startedAt).toLocaleString() : '—'} />
        </div>
      </div>

      {/* ── Security + Connectors ── */}
      <div className="grid grid-cols-1 xl:grid-cols-2 gap-5">
        {/* Security */}
        <div className="card p-6">
          <h2 className="text-sm font-semibold text-gray-800 mb-4">Security</h2>
          <div className="space-y-3">
            <SecurityRow label="CSRF Protection" status="active" />
            <SecurityRow label="Session Security" status="active" />
            <SecurityRow label="Password Hashing" value="Argon2id" status="active" />
          </div>
        </div>

        {/* Connectors */}
        <div className="card p-6">
          <h2 className="text-sm font-semibold text-gray-800 mb-4">Connectors</h2>
          <div className="empty-state">
            <div className="w-12 h-12 rounded-xl bg-gray-100 flex items-center justify-center text-gray-400">
              <span className="text-2xl">🔌</span>
            </div>
            <p className="text-sm font-semibold text-gray-700">No connectors configured</p>
            <p className="text-xs text-gray-400 max-w-[300px]">
              Add connectors in the connectors tab to manage resources from Docker, Pterodactyl, Proxmox and more.
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}

function InfoRow({ label, value }: { label: string; value: string }) {
  return (
    <div>
      <dt className="text-[11px] font-semibold text-gray-400 uppercase tracking-wider mb-1">
        {label}
      </dt>
      <dd className="text-[14px] text-gray-800 font-mono">{value}</dd>
    </div>
  );
}

function SecurityRow({
  label,
  value,
  status,
}: {
  label: string;
  value?: string;
  status: 'active' | 'inactive';
}) {
  return (
    <div className="flex items-center justify-between py-1">
      <span className="text-sm text-gray-600">{label}</span>
      <div className="flex items-center gap-2">
        {value && <span className="text-xs text-gray-500 font-mono">{value}</span>}
        <span
          className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-[11px] font-semibold ${
            status === 'active'
              ? 'bg-green-100 text-green-700'
              : 'bg-gray-100 text-gray-500'
          }`}
        >
          {status === 'active' ? 'Active' : 'Inactive'}
        </span>
      </div>
    </div>
  );
}
