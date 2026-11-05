import type { LucideProps } from 'lucide-react';

interface MetricCardProps {
  label: string;
  value: string;
  icon: React.ComponentType<LucideProps>;
  iconColor: string;
}

export function MetricCard({ label, value, icon: Icon, iconColor }: MetricCardProps) {
  return (
    <div className="card p-4">
      <div className="flex items-center gap-3 mb-3">
        <div className="grid h-9 w-9 place-items-center rounded-lg bg-bg-tertiary">
          <Icon className={cx('h-4 w-4', iconColor)} />
        </div>
        <span className="text-[11px] font-semibold uppercase tracking-[0.1em] text-text-muted">
          {label}
        </span>
      </div>
      <span className="heading-md">{value}</span>
    </div>
  );
}

function cx(...classes: (string | undefined | false | null)[]) {
  return classes.filter(Boolean).join(' ');
}
