import type { ReactNode } from 'react';

interface PageHeaderProps {
  eyebrow: string;
  title: string;
  lead?: string;
  actions?: ReactNode;
}

/** Consistent page title block with an editorial display face. */
export function PageHeader({ eyebrow, title, lead, actions }: PageHeaderProps) {
  return (
    <header className="rise flex flex-col gap-6 border-b border-line pb-7 md:flex-row md:items-end md:justify-between">
      <div className="min-w-0">
        <p className="eyebrow">{eyebrow}</p>
        <h1 className="display mt-3 text-[38px] sm:text-[46px]">{title}</h1>
        {lead && <p className="mt-3 max-w-2xl text-[14px] leading-relaxed text-paper-dim">{lead}</p>}
      </div>
      {actions && <div className="flex shrink-0 items-center gap-2">{actions}</div>}
    </header>
  );
}
