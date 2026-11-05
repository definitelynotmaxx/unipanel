interface InfoCardProps {
  label: string;
  value: string;
}

export function InfoCard({ label, value }: InfoCardProps) {
  return (
    <div className="flex flex-col py-1">
      <span className="text-[10px] font-semibold uppercase tracking-[0.12em] text-text-muted">
        {label}
      </span>
      <span className="text-[14px] text-text-primary">{value}</span>
    </div>
  );
}
