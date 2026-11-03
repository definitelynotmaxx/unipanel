import { useNavigate, type ErrorComponentRoutes } from '@tanstack/react-router';

import { AuthLayout } from '@/components/AuthLayout';
import { IconSearch } from '@/components/icons';
import { t } from '@/lib/i18n';

export function NotFoundPage() {
  const navigate = useNavigate();
  return (
    <AuthLayout
      eyebrow={t('error.notFound.eyebrow')}
      headline={t('error.notFound.title')}
      lead={t('error.notFound.lead')}
    >
      <div className="flex flex-col items-center gap-6">
        <div className="grid h-20 w-20 place-items-center rounded border border-line bg-ink-1/50">
          <IconSearch width={32} height={32} className="text-muted" />
        </div>
        <div className="flex gap-3">
          <button onClick={() => void navigate({ to: '/' })} className="btn btn-primary">
            {t('app.back')}
          </button>
          <button onClick={() => history.back()} className="btn btn-secondary">
            {t('error.notFound.back')}
          </button>
        </div>
      </div>
    </AuthLayout>
  );
}