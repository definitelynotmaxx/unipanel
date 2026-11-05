import { useQuery } from '@tanstack/react-query';
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

  return (
    <div className="space-y-5 animate-in">
      {/* ── Title ── */}
      <div>
        <h1 className="heading-lg">Dashboard</h1>
        <p className="text-sm text-gray-500 mt-1">
          Instance: {instanceName} · {version}
        </p>
      </div>

      {/* ── Top stats row ── */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <StatCard
          icon={<span className="text-xl">📦</span>}
          label="Version"
          value={version}
        />
        <StatCard
          icon={<span className="text-xl">{dbOk ? '✅' : '❌'}</span>}
          label="Database"
          value={dbOk ? 'Connected' : 'Disconnected'}
          color={dbOk ? 'green' : 'red'}
        />
        <StatCard
          icon={<span className="text-xl">⚙️</span>}
          label="Schema"
          value={schemaVersion != null ? `v${schemaVersion}` : '—'}
        />
        <StatCard
          icon={<span className="text-xl">🗄️</span>}
          label="Database"
          value={database}
        />
      </div>

      {/* ── Main content + System info ── */}
      <div className="grid grid-cols-1 xl:grid-cols-3 gap-5">
        {/* Left: Status + Monitoring */}
        <div className="xl:col-span-2 space-y-5">
          {/* Status section */}
          <div className="card p-5">
            <h2 className="text-sm font-semibold text-gray-800 mb-4">Status</h2>
            <div className="grid grid-cols-2 sm:grid-cols-4 gap-6">
              <Gauge value={dbOk ? 100 : 0} label="Database" color="green" />
              <Gauge value={bootstrap.data?.authenticated ? 100 : 50} label="Auth" color={bootstrap.data?.authenticated ? 'green' : 'yellow'} />
              <Gauge value={info.data?.uptime_seconds && info.data.uptime_seconds > 0 ? 100 : 0} label="Uptime" color="blue" />
              <Gauge value={100} label="System" color="blue" />
            </div>
          </div>

          {/* Monitoring */}
          <div className="card p-5">
            <div className="flex items-center justify-between mb-4">
              <h2 className="text-sm font-semibold text-gray-800">Monitoring</h2>
              <div className="flex gap-2">
                <button className="btn btn-sm bg-blue-50 text-blue-600 border-blue-200">CPU</button>
                <button className="btn btn-sm">Memory</button>
                <button className="btn btn-sm">Network</button>
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
        <div className="card p-5">
          <h2 className="text-sm font-semibold text-gray-800 mb-4">System Information</h2>
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
            <h3 className="text-xs font-semibold text-gray-500 uppercase tracking-wider mb-3">
              Applications
            </h3>
            <div className="space-y-2">
              <EmptyAppCard label="No apps configured" desc="Connect resources to see them here." />
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
  color?: 'green' | 'red' | 'blue' | 'yellow';
}) {
  const colorMap: Record<string, string> = {
    green: 'bg-green-50 border-green-200',
    red: 'bg-red-50 border-red-200',
    blue: 'bg-blue-50 border-blue-200',
    yellow: 'bg-yellow-50 border-yellow-200',
  };

  return (
    <div className={`stat-card ${color ? colorMap[color] : ''}`}>
      <div className="flex items-center gap-2 mb-2">
        <span>{icon}</span>
        <span className="text-[11px] font-medium text-gray-500 uppercase tracking-wider">
          {label}
        </span>
      </div>
      <span className="text-[22px] font-bold">{value}</span>
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
    yellow: '#f59e0b',
    red: '#ef4444',
  };
  const strokeColor = colorMap[color] || '#3b82f6';

  return (
    <div className="gauge">
      <div className="relative">
        <svg width={radius * 2 + stroke * 2} height={radius * 2 + stroke * 2}>
          <circle
            cx={radius + stroke}
            cy={radius + stroke}
            r={radius}
            fill="none"
            stroke="#f0f0f0"
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
          <span className="text-[13px] font-bold text-gray-800">{normalized}%</span>
        </div>
      </div>
      <span className="text-xs text-gray-500 font-medium">{label}</span>
    </div>
  );
}

function SystemInfoRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex justify-between items-baseline">
      <dt className="text-xs text-gray-500 font-medium">{label}</dt>
      <dd className="text-xs text-gray-700 font-mono">{value}</dd>
    </div>
  );
}

function EmptyAppCard({ label, desc }: { label: string; desc: string }) {
  return (
    <div className="empty-state py-3">
      <div className="w-10 h-10 rounded-lg bg-gray-100 flex items-center justify-center text-gray-400">
        <span className="text-lg">📋</span>
      </div>
      <p className="text-sm font-semibold text-gray-700">{label}</p>
      <p className="text-xs text-gray-400">{desc}</p>
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
