import { useQuery } from '@tanstack/react-query';
import {
  IconClock,
  IconDatabase,
  IconGlobe,
  IconKey,
  IconPlug,
} from '@/components/icons';

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
    <div className="space-y-6">
      {/* Title */}
      <div>
        <h1 className="text-2xl font-bold text-gray-900">Administration</h1>
        <p className="text-sm text-gray-500 mt-1">
          Instance settings, security, and connectors
        </p>
      </div>

      {/* Instance info */}
      <div className="bg-white rounded-xl border border-gray-200 p-6">
        <h2 className="text-sm font-semibold text-gray-800 mb-5">Instance</h2>
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-x-8 gap-y-4">
          <InfoRow
            icon={<IconGlobe className="w-4 h-4" />}
            label="Name"
            value={instanceName}
          />
          <InfoRow
            icon={<IconGlobe className="w-4 h-4" />}
            label="Language"
            value={locale}
          />
          <InfoRow
            icon={<IconKey className="w-4 h-4" />}
            label="Version"
            value={version}
          />
          <InfoRow
            icon={<IconDatabase className="w-4 h-4" />}
            label="Schema"
            value={schemaVersion != null ? `v${schemaVersion}` : '—'}
          />
          <InfoRow
            icon={<IconDatabase className="w-4 h-4" />}
            label="Database"
            value={database}
          />
          <InfoRow
            icon={<IconClock className="w-4 h-4" />}
            label="Started"
            value={startedAt ? new Date(startedAt).toLocaleString() : '—'}
          />
        </div>
      </div>

      {/* Security + Connectors */}
      <div className="grid grid-cols-1 xl:grid-cols-2 gap-6">
        {/* Security */}
        <div className="bg-white rounded-xl border border-gray-200 p-6">
          <h2 className="text-sm font-semibold text-gray-800 mb-5">Security</h2>
          <div className="space-y-3">
            <SecurityRow label="CSRF Protection" status="active" />
            <SecurityRow label="Session Security" status="active" />
            <SecurityRow label="Password Hashing" value="Argon2id" status="active" />
          </div>
        </div>

        {/* Connectors */}
        <div className="bg-white rounded-xl border border-gray-200 p-6">
          <h2 className="text-sm font-semibold text-gray-800 mb-5">Connectors</h2>
          <div className="text-center py-8">
            <div className="w-12 h-12 rounded-xl bg-gray-100 flex items-center justify-center mx-auto mb-4">
              <IconPlug className="w-6 h-6 text-gray-400" />
            </div>
            <p className="text-sm font-semibold text-gray-700 mb-2">No connectors configured</p>
            <p className="text-xs text-gray-400 max-w-[300px] mx-auto">
              Add connectors to manage resources from Docker, Pterodactyl, Proxmox and more.
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}

function InfoRow({
  icon,
  label,
  value,
}: {
  icon: React.ReactNode;
  label: string;
  value: string;
}) {
  return (
    <div className="flex items-start gap-3">
      <div className="w-8 h-8 rounded-lg bg-gray-100 flex items-center justify-center text-gray-500 mt-0.5">
        {icon}
      </div>
      <div>
        <dt className="text-[11px] font-semibold text-gray-400 uppercase tracking-wider mb-1">
          {label}
        </dt>
        <dd className="text-[14px] text-gray-800 font-mono">{value}</dd>
      </div>
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
    <div className="flex items-center justify-between py-2">
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
