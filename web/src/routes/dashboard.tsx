import { useQuery } from '@tanstack/react-query';
import {
  IconBox,
  IconDatabase,
  IconLayoutDashboard,
  IconServer,
  IconSettings,
} from '@/components/icons';
import { bootstrapQuery, infoQuery, healthQuery } from '@/lib/queries';

export function DashboardPage() {
  const bootstrap = useQuery(bootstrapQuery);
  const info = useQuery({ ...infoQuery, enabled: !!bootstrap.data?.authenticated });
  const health = useQuery(healthQuery);

  const version = bootstrap.data?.version ?? '—';
  const instanceName = bootstrap.data?.instance_name ?? 'UniPanel';
  const database = info.data?.database ?? '—';
  const schemaVersion = info.data?.schema_version;
  const startedAt = info.data?.started_at;
  const uptimeSeconds = info.data?.uptime_seconds ?? health.data?.uptime_seconds ?? 0;
  const dbOk = health.data?.database_ok ?? false;
  const authenticated = bootstrap.data?.authenticated ?? false;

  return (
    <div className="space-y-6">
      {/* Title */}
      <div>
        <h1 className="text-2xl font-bold text-gray-900">Dashboard</h1>
        <p className="text-sm text-gray-500 mt-1">
          Instance: {instanceName} · {version}
        </p>
      </div>

      {/* Stats row */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        <StatCard
          icon={<IconBox className="w-5 h-5" />}
          label="Version"
          value={version}
          color="blue"
        />
        <StatCard
          icon={<IconDatabase className="w-5 h-5" />}
          label="Database"
          value={dbOk ? 'Connected' : 'Disconnected'}
          color={dbOk ? 'green' : 'red'}
        />
        <StatCard
          icon={<IconSettings className="w-5 h-5" />}
          label="Schema"
          value={schemaVersion != null ? `v${schemaVersion}` : '—'}
          color="purple"
        />
        <StatCard
          icon={<IconServer className="w-5 h-5" />}
          label="Auth"
          value={authenticated ? 'Active' : 'Inactive'}
          color={authenticated ? 'green' : 'amber'}
        />
      </div>

      {/* Main content with tabs */}
      <div className="grid grid-cols-1 xl:grid-cols-3 gap-6">
        {/* Left: Status + Monitoring */}
        <div className="xl:col-span-2 space-y-6">
          {/* Status tab */}
          <div className="bg-white rounded-xl border border-gray-200 p-6">
            <h2 className="text-sm font-semibold text-gray-800 mb-5">Status</h2>
            <div className="grid grid-cols-2 sm:grid-cols-4 gap-6">
              <Gauge value={dbOk ? 100 : 0} label="Database" color="green" />
              <Gauge value={authenticated ? 100 : 50} label="Auth" color={authenticated ? 'green' : 'amber'} />
              <Gauge value={uptimeSeconds > 0 ? 100 : 0} label="Uptime" color="blue" />
              <Gauge value={100} label="System" color="blue" />
            </div>
          </div>

          {/* Monitoring tab */}
          <div className="bg-white rounded-xl border border-gray-200 p-6">
            <div className="flex items-center justify-between mb-5">
              <h2 className="text-sm font-semibold text-gray-800">Monitoring</h2>
              <div className="flex gap-1.5">
                <button className="px-3 py-1.5 text-xs font-medium rounded-md bg-blue-50 text-blue-600 border border-blue-200">
                  CPU
                </button>
                <button className="px-3 py-1.5 text-xs font-medium rounded-md bg-white text-gray-600 border border-gray-200 hover:bg-gray-50">
                  Memory
                </button>
                <button className="px-3 py-1.5 text-xs font-medium rounded-md bg-white text-gray-600 border border-gray-200 hover:bg-gray-50">
                  Network
                </button>
              </div>
            </div>
            <div className="h-48 flex items-end justify-between gap-1 px-2">
              {Array.from({ length: 40 }).map((_, i) => {
                const h = 20 + Math.sin(i * 0.4) * 30 + Math.random() * 30;
                return (
                  <div
                    key={i}
                    className="flex-1 rounded-t bg-gradient-to-t from-blue-500/80 to-blue-400/40 transition-all"
                    style={{ height: `${h}%` }}
                  />
                );
              })}
            </div>
          </div>
        </div>

        {/* Right: System info */}
        <div className="bg-white rounded-xl border border-gray-200 p-6">
          <h2 className="text-sm font-semibold text-gray-800 mb-5">System Information</h2>
          <dl className="space-y-3 text-sm">
            <SystemInfoRow label="Hostname" value={instanceName} />
            <SystemInfoRow label="Version" value={version} />
            <SystemInfoRow label="Database" value={database} />
            <SystemInfoRow label="Schema" value={schemaVersion != null ? `v${schemaVersion}` : '—'} />
            <SystemInfoRow label="Uptime" value={formatUptime(uptimeSeconds)} />
            {startedAt && (
              <SystemInfoRow label="Started" value={new Date(startedAt).toLocaleString()} />
            )}
          </dl>

          {/* Applications section */}
          <div className="mt-6 pt-4 border-t border-gray-100">
            <h3 className="text-xs font-semibold text-gray-500 uppercase tracking-wider mb-4">
              Applications
            </h3>
            <div className="space-y-2">
              <EmptyAppCard />
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function StatCard({
  icon,
  label,
  value,
  color,
}: {
  icon: React.ReactNode;
  label: string;
  value: string;
  color: string;
}) {
  const colorMap: Record<string, string> = {
    green: 'bg-green-50 border-green-200 text-green-700',
    red: 'bg-red-50 border-red-200 text-red-700',
    blue: 'bg-blue-50 border-blue-200 text-blue-700',
    amber: 'bg-amber-50 border-amber-200 text-amber-700',
    purple: 'bg-purple-50 border-purple-200 text-purple-700',
  };

  const style = colorMap[color] || colorMap.blue;

  return (
    <div className={`rounded-xl border p-5 transition-shadow hover:shadow-sm ${(style || '').split(' ').slice(2).join(' ')}`}>
      <div className="flex items-center gap-3 mb-3">
        <div className={`w-9 h-9 rounded-lg flex items-center justify-center ${style}`}>
          {icon}
        </div>
        <span className="text-[11px] font-semibold text-gray-500 uppercase tracking-wider">
          {label}
        </span>
      </div>
      <span className="text-2xl font-bold text-gray-900">{value}</span>
    </div>
  );
}

function Gauge({ value, label, color }: { value: number; label: string; color: string }) {
  const radius = 36;
  const stroke = 5;
  const normalized = Math.min(100, Math.max(0, value));
  const circumference = 2 * Math.PI * radius;
  const offset = circumference - (normalized / 100) * circumference;

  const colorMap: Record<string, string> = {
    green: '#10b981',
    blue: '#3b82f6',
    amber: '#f59e0b',
    red: '#ef4444',
  };
  const strokeColor = colorMap[color] || '#3b82f6';

  return (
    <div className="flex flex-col items-center">
      <div className="relative">
        <svg width={radius * 2 + stroke * 2} height={radius * 2 + stroke * 2}>
          <circle
            cx={radius + stroke}
            cy={radius + stroke}
            r={radius}
            fill="none"
            stroke="#f3f4f6"
            strokeWidth={stroke}
          />
          <circle
            cx={radius + stroke}
            cy={radius + stroke}
            r={radius}
            fill="none"
            stroke={strokeColor}
            strokeWidth={stroke}
            strokeDasharray={circumference}
            strokeDashoffset={offset}
            strokeLinecap="round"
            style={{ transition: 'stroke-dashoffset 1s ease-out' }}
          />
        </svg>
        <div className="absolute inset-0 flex items-center justify-center">
          <span className="text-sm font-bold text-gray-900">{normalized}%</span>
        </div>
      </div>
      <span className="text-xs text-gray-500 font-medium mt-2">{label}</span>
    </div>
  );
}

function SystemInfoRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex justify-between items-baseline">
      <dt className="text-xs text-gray-500 font-medium">{label}</dt>
      <dd className="text-xs text-gray-800 font-mono">{value}</dd>
    </div>
  );
}

function EmptyAppCard() {
  return (
    <div className="text-center py-6">
      <div className="w-10 h-10 rounded-xl bg-gray-100 flex items-center justify-center mx-auto mb-3">
        <IconLayoutDashboard className="w-5 h-5 text-gray-400" />
      </div>
      <p className="text-sm font-semibold text-gray-700">No apps configured</p>
      <p className="text-xs text-gray-400 mt-1">Connect resources to see them here.</p>
    </div>
  );
}

function formatUptime(seconds: number): string {
  const d = Math.floor(seconds / 86400);
  const h = Math.floor((seconds % 86400) / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  if (d > 0) return `${d}d ${String(h).padStart(2, '0')}h`;
  if (h > 0) return `${h}h ${String(m).padStart(2, '0')}m`;
  return `${m}m`;
}
