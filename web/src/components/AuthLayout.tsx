import { readBoot } from '@/lib/boot';

interface AuthLayoutProps {
  eyebrow: string;
  headline: React.ReactNode;
  lead: string;
  children: React.ReactNode;
  footer?: React.ReactNode;
}

export function AuthLayout({ eyebrow, headline, lead, children }: AuthLayoutProps) {
  const boot = readBoot();

  return (
    <div className="min-h-screen flex bg-[#f5f6f8]">
      {/* Left panel — branding */}
      <div className="hidden lg:flex lg:w-[45%] xl:w-[50%] flex-col justify-between p-12 relative overflow-hidden">
        {/* Background gradient */}
        <div className="absolute inset-0 bg-gradient-to-br from-blue-600 to-indigo-700" />
        <div className="absolute inset-0 opacity-10"
          style={{
            backgroundImage: 'url("data:image/svg+xml,%3Csvg width=\'60\' height=\'60\' xmlns=\'http://www.w3.org/2000/svg\'%3E%3Cdefs%3E%3Cpattern id=\'grid\' width=\'60\' height=\'60\' patternUnits=\'userSpaceOnUse\'%3E%3Cpath d=\'M 60 0 L 0 0 0 60\' fill=\'none\' stroke=\'white\' stroke-width=\'0.5\'/%3E%3C/pattern%3E%3C/defs%3E%3Crect width=\'100%25\' height=\'100%25\' fill=\'url(%23grid)\'/%3E%3C/svg%3E")',
          }}
        />

        <div className="relative z-10">
          <div className="flex items-center gap-3 mb-2">
            <div className="w-8 h-8 rounded-lg bg-white/20 flex items-center justify-center">
              <span className="text-white font-bold text-sm">UP</span>
            </div>
            <span className="text-white/80 text-[10px] font-bold tracking-[0.2em] uppercase">
              UniPanel
            </span>
          </div>
        </div>

        <div className="relative z-10">
          <p className="text-white/50 text-[10px] font-semibold uppercase tracking-[0.2em] mb-3">
            {eyebrow}
          </p>
          <h1 className="text-white text-4xl font-semibold leading-tight mb-4">
            {headline}
          </h1>
          <p className="text-white/70 text-base leading-relaxed max-w-md">
            {lead}
          </p>
        </div>

        <div className="relative z-10 text-white/40 text-[8px] font-semibold tracking-[0.2em] uppercase">
          {boot.instanceName} · v{boot.version} · MIT
        </div>
      </div>

      {/* Right panel — form */}
      <div className="flex flex-1 flex-col">
        {/* Mobile header */}
        <div className="h-14 flex items-center px-6 border-b border-gray-200 bg-white lg:hidden">
          <div className="flex items-center gap-2">
            <div className="w-7 h-7 rounded bg-blue-600 flex items-center justify-center">
              <span className="text-white text-[10px] font-bold">UP</span>
            </div>
            <span className="text-[10px] font-bold tracking-[0.15em] text-gray-500 uppercase">
              UniPanel
            </span>
          </div>
        </div>

        <div className="flex flex-1 items-center justify-center p-6 sm:p-10">
          <div className="w-full max-w-[420px]">
            {children}
          </div>
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
}: {
  eyebrow: string;
  title: React.ReactNode;
  lead?: string;
}) {
  return (
    <div className="mb-7">
      <p className="text-[10px] font-semibold text-gray-400 uppercase tracking-[0.16em] mb-2">
        {eyebrow}
      </p>
      <h2 className="text-[26px] font-semibold text-gray-800 leading-tight mb-2">
        {title}
      </h2>
      {lead && <p className="text-[13px] text-gray-500 leading-relaxed">{lead}</p>}
    </div>
  );
}
