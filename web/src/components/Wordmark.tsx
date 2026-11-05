import { readBoot } from '@/lib/boot';
import { cx } from '@/lib/cx';

interface WordmarkProps {
  /** Hides the text lockup and shows only the glyph. */
  glyphOnly?: boolean;
  /** Extra line under the name, hidden when compact. */
  subtitle?: string;
  className?: string;
}

/** The one place the product lockup is drawn. */
export function Wordmark({ glyphOnly = false, subtitle, className }: WordmarkProps) {
  const boot = readBoot();
  return (
    <span className={cx('inline-flex items-center gap-3 select-none', className)}>
      <Glyph />
      {!glyphOnly && (
        <span className="flex flex-col leading-none">
          <span
            className="tracking-[0.08em] text-[17px] font-medium text-text-primary"
            style={{ fontFamily: 'var(--font-display)' }}
          >
            UNIPANEL
          </span>
          <span className="text-[9px] font-semibold uppercase tracking-[0.18em] text-text-muted mt-[3px]">
            {subtitle ?? `v${boot.version}`}
          </span>
        </span>
      )}
    </span>
  );
}

/** Geometric mark: a square frame with a U and a sodium indicator. */
export function Glyph({ size = 30 }: { size?: number }) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 32 32"
      aria-hidden="true"
      className="shrink-0"
    >
      <rect x="0.5" y="0.5" width="31" height="31" fill="none" stroke="rgba(228,232,239,0.15)" />
      <path
        d="M9 8v10.5a4.5 4.5 0 0 0 4.5 4.5h5a4.5 4.5 0 0 0 4.5-4.5V8"
        fill="none"
        stroke="currentColor"
        strokeWidth="1.7"
        style={{ color: 'var(--color-text-primary)' }}
      />
      <rect x="22.5" y="4.5" width="5" height="5" fill="var(--color-accent)" />
    </svg>
  );
}
