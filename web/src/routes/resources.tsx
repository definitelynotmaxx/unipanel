import { useNavigate } from '@tanstack/react-router';

import { EmptyState } from '@/components/EmptyState';
import { t } from '@/lib/i18n';
import { bootstrapQuery } from '@/lib/queries';
import { useQuery } from '@tanstack/react-query';

export function ResourcesPage() {
  const navigate = useNavigate();
  const bootstrap = useQuery(bootstrapQuery);

  const authenticated = bootstrap.data?.authenticated ?? false;

  return (
    <div className="space-y-6 animate-in">
      <div className="section-header">
        <div>
          <p className="text-[10px] font-semibold uppercase tracking-[0.16em] text-text-muted mb-1">
            {t('resources.eyebrow')}
          </p>
          <h1 className="heading-lg">{t('resources.title')}</h1>
        </div>
      </div>

      <div className="card">
        {authenticated ? (
          <div className="p-6">
            <EmptyState
              icon="Server"
              title={t('resources.empty.title')}
              description={t('resources.empty.lead')}
              actionLabel={t('dashboard.connect.cta')}
              onAction={() => navigate({ to: '/admin' })}
            />
          </div>
        ) : (
          <div className="p-6">
            <EmptyState
              icon="Server"
              title="No servers configured"
              description="Connect to a resource provider like Docker or Pterodactyl in Administration to see server inventory here."
            />
          </div>
        )}
      </div>
    </div>
  );
}
