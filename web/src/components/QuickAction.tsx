import type { LucideProps } from 'lucide-react';

interface QuickActionProps {
  label: string;
  description: string;
  icon: React.ComponentType<LucideProps>;
  onClick: () => void;
}

export function QuickAction({ label, description, icon: Icon, onClick }: QuickActionProps) {
  return (
    <button
      type="button"
      className="btn w-full justify-start"
      onClick={onClick}
    >
      <Icon className="h-4 w-4 shrink-0 opacity-60" />
      <div className="flex-1 text-left">
        <span className="text-[13px] font-medium">{label}</span>
        <span className="block text-[11px] text-text-muted">{description}</span>
      </div>
    </button>
  );
}
