import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { Link, Outlet, useLocation, useNavigate } from '@tanstack/react-router';
import { useEffect } from 'react';

import { StatusPill } from '@/components/StatusPill';
import { Wordmark } from '@/components/Wordmark';
import {
  IconChevronLeft,
  IconChevronRight,
  IconGrid,
  IconLogout,
  IconMenu,
  IconServer,
  IconSettings,
} from '@/components/icons';
import { api } from '@/lib/api';
import { cx } from '@/lib/cx';
import { bootLocale } from '@/lib/boot';
import { formatUptime } from '@/lib/format';
import { t } from '@/lib/i18n';
import { bootstrapQuery, infoQuery } from '@/lib/queries';
import { useUi } from '@/stores/ui';

interface NavItem {
  to: string;
  labelKey: string;
  icon: typeof IconGrid;
  exact?: boolean;
}

const NAV: NavItem[] = [
  { to: '/', labelKey: 'nav.dashboards', icon: IconGrid, exact: true },
  { to: '/resources', labelKey: 'nav.resources', icon: IconServer },
  { to: '/admin', labelKey: 'nav.admin', icon: IconSettings },
];

function initials(name: string): string {
  const parts = name.trim().split(/\s+/).filter(Boolean);
  if (parts.length === 0) return 'U';
  if (parts.length === 1) return parts[0]!.slice(0, 2).toUpperCase();
  return `${parts[0]![0]}${parts[parts.length - 1]![0]}`.toUpperCase();
}

export function AppShell() {
  const collapsed = useUi((s) => s.sidebarCollapsed);
  const toggleSidebar = useUi((s) => s.toggleSidebar);
  const mobileNavOpen = useUi((s) => s.mobileNavOpen);
  const setMobileNavOpen = useUi((s) => s.setMobileNavOpen);
  const location = useLocation();
  const navigate = useNavigate();
  const queryClient = useQueryClient();

  const bootstrap = useQuery(bootstrapQuery);
  const info = useQuery({
    ...infoQuery,
    enabled: !!bootstrap.data?.authenticated,
  });
  const user = bootstrap.data?.user;

  const logout = useMutation({
    mutationFn: () => api.logout(),
    onSuccess: () => {
      queryClient.clear();
      void navigate({ to: '/login', replace: true });
    },
  });

  useEffect(() => {
    setMobileNavOpen(false);
  }, [location.pathname, setMobileNavOpen]);

  const width = collapsed ? 'var(--rail-compact)' : 'var(--rail)';
  const instanceName = info.data?.instance_name ?? bootstrap.data?.instance_name ?? '';

  return (
    <div className="relative min-h-screen flex">
      {/* Mobile overlay */}
      {mobileNavOpen && (
        <div
          className="fixed inset-0 z-40 bg-black/50 backdrop-blur-sm lg:hidden"
          onClick={() => setMobileNavOpen(false)}
        />
      )}

      {/* ── Sidebar ── */}
      <aside
        style={{ width }}
        className={cx(
          'fixed inset-y-0 left-0 z-50 flex flex-col border-r border-border bg-bg-secondary transition-[width] duration-300 ease-out',
          'lg:sticky lg:top-0 lg:z-auto',
          mobileNavOpen ? 'translate-x-0' : '-translate-x-full',
        )}
      >
        {/* Logo bar */}
        <div className={cx(
          'flex h-14 items-center border-b border-border',
          collapsed ? 'justify-center px-2' : 'justify-between px-4',
        )}>
          {collapsed ? (
            <Wordmark glyphOnly />
          ) : (
            <Wordmark subtitle={t('app.tagline')} />
          )}
          <button
            type="button"
            onClick={toggleSidebar}
            className={cx(
              'btn btn-icon btn-ghost hidden lg:inline-flex',
              collapsed && 'hidden',
            )}
            aria-label={t('nav.collapse')}
          >
            <IconChevronLeft className="h-4 w-4" />
          </button>
        </div>

        {/* Navigation */}
        <nav className="flex-1 overflow-y-auto py-3" aria-label="Primary">
          <div className="space-y-0.5 px-2">
            {NAV.map((item) => {
              const Icon = item.icon;
              const active = item.exact
                ? location.pathname === item.to
                : location.pathname.startsWith(item.to);

              return (
                <Link
                  key={item.to}
                  to={item.to}
                  activeOptions={{ exact: !!item.exact }}
                  className={cx(
                    'nav-item',
                    active && 'nav-item-active',
                    collapsed && 'justify-center px-0',
                  )}
                  title={collapsed ? t(item.labelKey) : undefined}
                  onClick={() => setMobileNavOpen(false)}
                >
                  <Icon className="h-[15px] w-[15px] shrink-0 opacity-60" />
                  {!collapsed && (
                    <>
                      <span className="flex-1 truncate text-[13.5px]">{t(item.labelKey)}</span>
                    </>
                  )}
                </Link>
              );
            })}
          </div>

          {/* Quick stats */}
          {!collapsed && (
            <div className="mx-3 mt-4 rounded-lg border border-border bg-bg-tertiary/50 p-3">
              <div className="grid grid-cols-2 gap-3">
                <div>
                  <p className="text-[10px] font-semibold uppercase tracking-[0.12em] text-text-muted">
                    {t('shell.uptime')}
                  </p>
                  <p className="mono mt-0.5 text-[11px] text-text-secondary">
                    {formatUptime(info.data?.uptime_seconds ?? 0)}
                  </p>
                </div>
                <div>
                  <p className="text-[10px] font-semibold uppercase tracking-[0.12em] text-text-muted">
                    {t('dashboard.schematic.coreSub')}
                  </p>
                  <p className="mono mt-0.5 text-[11px] text-text-secondary">
                    v{bootstrap.data?.version ?? '—'}
                  </p>
                </div>
              </div>
            </div>
          )}
        </nav>

        {/* User bar */}
        <div className="border-t border-border p-3">
          <div className={cx('flex items-center gap-2.5', collapsed && 'justify-center')}>
            <span className="grid h-8 w-8 shrink-0 place-items-center rounded-md bg-bg-tertiary text-[11px] font-semibold tracking-wide text-text-primary">
              {initials(user?.display_name ?? '')}
            </span>
            {!collapsed && (
              <>
                <div className="min-w-0 flex-1">
                  <p className="truncate text-[13px] leading-tight text-text-primary">
                    {user?.display_name ?? '—'}
                  </p>
                  <p className="mono text-[9px] uppercase tracking-[0.16em] text-text-muted">
                    {user?.role ?? ''}
                  </p>
                </div>
                <button
                  type="button"
                  onClick={() => logout.mutate()}
                  disabled={logout.isPending}
                  aria-label={t('nav.logout')}
                  className="btn btn-icon btn-ghost"
                  title={t('nav.logout')}
                >
                  <IconLogout className="h-4 w-4" />
                </button>
              </>
            )}
          </div>
          {collapsed && (
            <button
              type="button"
              onClick={toggleSidebar}
              className="btn btn-ghost mt-2 h-8 w-full"
              aria-label={t('nav.expand')}
            >
              <IconChevronRight className="h-4 w-4" />
            </button>
          )}
          {!collapsed && (
            <p className="mono mt-3 flex items-center justify-between text-[8px] uppercase tracking-[0.16em] text-text-muted">
              <span>v{bootstrap.data?.version ?? '—'}</span>
              <span>{bootLocale()}</span>
            </p>
          )}
        </div>
      </aside>

      {/* ── Main content ── */}
      <div className="flex min-w-0 flex-1 flex-col">
        {/* Header */}
        <header className="sticky top-0 z-30 flex h-14 items-center justify-between gap-3 border-b border-border bg-bg-primary/80 px-4 backdrop-blur-md lg:px-6">
          <div className="flex items-center gap-3 min-w-0">
            <button
              type="button"
              onClick={() => setMobileNavOpen(true)}
              className="btn btn-icon btn-ghost lg:hidden"
              aria-label={t('nav.menu')}
            >
              <IconMenu className="h-4 w-4" />
            </button>
            <p className="mono truncate text-[9px] uppercase tracking-[0.2em] text-text-muted">
              {instanceName}
            </p>
          </div>
          <div className="flex items-center gap-2">
            <StatusPill />
          </div>
        </header>

        {/* Page content */}
        <main className="mx-auto w-full max-w-[1440px] flex-1 px-4 pb-12 pt-6 lg:px-6">
          <Outlet />
        </main>

        {/* Footer */}
        <footer className="border-t border-border px-4 py-3 lg:px-6">
          <p className="mono flex flex-wrap items-center gap-x-4 gap-y-1 text-[8px] uppercase tracking-[0.18em] text-text-muted">
            <span>{bootstrap.data?.instance_name ?? 'UniPanel'} · v{bootstrap.data?.version ?? '—'}</span>
            <span className="ml-auto">MIT</span>
          </p>
        </footer>
      </div>
    </div>
  );
}
