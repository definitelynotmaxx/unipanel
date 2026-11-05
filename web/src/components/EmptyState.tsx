import {
  Server,
  X as IconX,
  type LucideProps,
} from 'lucide-react';

interface EmptyStateProps {
  icon?: string;
  title: string;
  description?: string;
  actionLabel?: string;
  onAction?: () => void;
}

export function EmptyState({
  icon: iconProp = 'Server',
  title,
  description,
  actionLabel,
  onAction,
}: EmptyStateProps) {
  // Map icon name to lucide-react component
  const iconMap: Record<string, React.ComponentType<LucideProps>> = {
    Server,
    X: IconX,
  };
  const Icon = iconMap[iconProp] || Server;

  return (
    <div className="empty-state">
      <div className="empty-state-icon">
        <Icon className="h-5 w-5" />
      </div>
      <div>
        <p className="empty-state-title">{title}</p>
        {description && (
          <p className="empty-state-desc mt-1">{description}</p>
        )}
      </div>
      {actionLabel && onAction && (
        <button className="btn btn-primary" onClick={onAction}>
          {actionLabel}
        </button>
      )}
    </div>
  );
}
