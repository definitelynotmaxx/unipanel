import { t } from '@/lib/i18n';
import { Wordmark } from '@/components/Wordmark';
import { readBoot } from '@/lib/boot';

interface AuthLayoutProps {
  eyebrow: string;
  headline: React.ReactNode;
  lead: string;
  children: React.ReactNode;
  footer?: React.ReactNode;
}

export function AuthLayout({ eyebrow, headline, lead, children, footer }: AuthLayoutProps) {
  const boot = readBoot();

  return (
    <div className="relative min-h-screen flex">
      {/* Left panel — branding + info */}
      <div className="hidden lg:flex lg:w-[55%] xl:w-[50%] flex-col justify-between border-r border-border bg-bg-secondary p-12 relative overflow-hidden">
        {/* Decorative gradient */}
        <div className="absolute inset-0 bg-gradient-to-br from-bg-secondary via-bg-primary to-bg-secondary opacity-80" />
        <div className="absolute top-[-20%] right-[-10%] w-[500px] h-[500px] rounded-full opacity-30 blur-[120px]"
          style={{ background: 'radial-gradient(circle, rgba(255,90,43,0.12) 0%, transparent 70%)' }}
        />

        <div className="relative z-10">
          <Wordmark subtitle={t('app.tagline')} />
        </div>

        <div className="relative z-10 max-w-lg">
          <p className="text-[10px] font-semibold uppercase tracking-[0.2em] text-text-muted mb-4">
            {eyebrow}
          </p>
          <h1 className="heading-xl text-5xl xl:text-6xl mb-5">{headline}</h1>
          <p className="text-base leading-relaxed text-text-secondary max-w-md">
            {lead}
          </p>
        </div>

        <div className="relative z-10 flex items-center gap-5 text-[8px] font-semibold uppercase tracking-[0.2em] text-border">
          <span>{boot.instanceName} · v{boot.version}</span>
          <span className="ml-auto">MIT</span>
        </div>
      </div>

      {/* Right panel — form */}
      <div className="flex flex-1 flex-col">
        {/* Mobile header */}
        <div className="flex h-14 items-center border-b border-border bg-bg-secondary px-6 lg:hidden">
          <Wordmark subtitle={t('app.tagline')} />
        </div>

        <div className="flex flex-1 items-center justify-center p-6 sm:p-10">
          <div className="w-full max-w-[420px]">
            {children}
          </div>
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between gap-4 px-6 py-4">
          <span className="mono text-[8px] uppercase tracking-[0.2em] text-text-muted">
            {t('app.name')} · {boot.instanceName}
          </span>
          {footer}
        </div>
      </div>
    </div>
  );
}

/** Shared heading block for the form column. */
export function AuthHeading({
  eyebrow,
  title,
  lead,
  aside,
}: {
  eyebrow: string;
  title: React.ReactNode;
  lead?: string;
  aside?: React.ReactNode;
}) {
  return (
    <div className="mb-8">
      <div className="flex items-center justify-between mb-3">
        <p className="text-[10px] font-semibold uppercase tracking-[0.16em] text-text-muted">
          {eyebrow}
        </p>
        {aside}
      </div>
      <h2 className="heading-lg text-[28px] mb-2">{title}</h2>
      {lead && <p className="text-[13px] leading-relaxed text-text-secondary">{lead}</p>}
    </div>
  );
}
