import type { ReactNode } from 'react';

import { Backdrop } from '@/components/Backdrop';
import { Schematic } from '@/components/Schematic';
import { Wordmark } from '@/components/Wordmark';
import { bootLocale, readBoot } from '@/lib/boot';
import { t } from '@/lib/i18n';

interface AuthLayoutProps {
  eyebrow: string;
  headline: ReactNode;
  lead: string;
  children: ReactNode;
  footer?: ReactNode;
}

/**
 * Split screen used by login and the setup wizard.
 *
 * On desktop the left half is a quiet editorial statement with the architecture
 * diagram; the form lives on the right with a hairline divider, like a piece of
 * rack hardware.
 */
export function AuthLayout({ eyebrow, headline, lead, children, footer }: AuthLayoutProps) {
  const boot = readBoot();
  return (
    <div className="relative min-h-screen">
      <Backdrop />

      <div className="relative z-10 grid min-h-screen lg:grid-cols-[1.1fr_minmax(420px,0.9fr)]">
        <section className="relative hidden flex-col justify-between overflow-hidden border-r border-line px-12 py-10 lg:flex">
          <span
            className="mono pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-[8px] uppercase tracking-[0.4em] text-line-strong"
            style={{ writingMode: 'vertical-rl', transform: 'translateY(-50%) rotate(180deg)' }}
            aria-hidden="true"
          >
            {t('app.name')} / {t('app.tagline')} / M0
          </span>

          <header className="rise">
            <Wordmark subtitle={t('app.tagline')} />
          </header>

          <div className="max-w-[560px] py-10">
            <p className="eyebrow rise" style={{ animationDelay: '80ms' }}>
              {eyebrow}
            </p>
            <h1
              className="display rise mt-5 text-[clamp(2.9rem,4.6vw,4.5rem)] text-paper"
              style={{ animationDelay: '140ms' }}
            >
              {headline}
            </h1>
            <p
              className="rise mt-6 max-w-[430px] text-[14.5px] leading-relaxed text-paper-dim"
              style={{ animationDelay: '200ms' }}
            >
              {lead}
            </p>

            <div className="rise mt-10 max-w-[520px]" style={{ animationDelay: '280ms' }}>
              <Schematic />
            </div>
          </div>

          <footer
            className="rise flex items-center gap-5 border-t border-line pt-5"
            style={{ animationDelay: '340ms' }}
          >
            <span className="mono text-[8px] uppercase tracking-[0.2em] text-muted">
              {boot.instanceName} · v{boot.version}
            </span>
            <span className="mono text-[8px] uppercase tracking-[0.2em] text-muted">
              {bootLocale()}
            </span>
            <span className="mono ml-auto text-[8px] uppercase tracking-[0.2em] text-line-strong">
              MIT
            </span>
          </footer>
        </section>

        <section className="flex min-h-screen flex-col px-5 py-8 sm:px-10 lg:px-14">
          <header className="rise flex items-center justify-between lg:hidden">
            <Wordmark subtitle={t('app.tagline')} />
          </header>

          <div className="flex flex-1 items-center justify-center py-10">
            <div className="rise w-full max-w-[420px]" style={{ animationDelay: '60ms' }}>
              {children}
            </div>
          </div>

          <footer className="flex items-center justify-between gap-4">
            <span className="mono text-[8px] uppercase tracking-[0.2em] text-muted">
              {t('app.name')} · {boot.instanceName}
            </span>
            {footer}
          </footer>
        </section>
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
  title: string;
  lead?: string;
  aside?: ReactNode;
}) {
  return (
    <div className="mb-7">
      <div className="flex items-center justify-between">
        <p className="eyebrow">{eyebrow}</p>
        {aside}
      </div>
      <h2 className="display mt-4 text-[30px] text-paper">{title}</h2>
      {lead && <p className="mt-2 text-[13.5px] leading-relaxed text-paper-dim">{lead}</p>}
    </div>
  );
}
