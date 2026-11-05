import { useNavigate } from '@tanstack/react-router';

import { EmptyState } from '@/components/EmptyState';

export function NotFoundPage() {
  const navigate = useNavigate();
  return (
    <div className="min-h-[60vh] flex items-center justify-center">
      <EmptyState
        icon="X"
        title="Page not found"
        description="The route does not exist. It may have moved with a later milestone."
        actionLabel="Back to dashboards"
        onAction={() => void navigate({ to: '/' })}
      />
    </div>
  );
}
