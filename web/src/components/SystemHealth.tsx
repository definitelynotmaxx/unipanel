interface ItemProps {
  label: string;
  value: string;
}

export function SystemHealth({ children }: { children: React.ReactNode }) {
  return <div className="flex flex-col gap-4">{children}</div>;
}

SystemHealth.Item = function Item({ label, value }: ItemProps) {
  return (
    <div className="flex flex-col">
      <span className="text-[10px] font-semibold uppercase tracking-[0.1em] text-text-muted">
        {label}
      </span>
      <span className="text-[13px] text-text-primary">{value}</span>
    </div>
  );
};
