import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { Link, Outlet, useLocation, useNavigate } from '@tanstack/react-router';
import { useEffect } from 'react';

import { Backdrop } from '@/components/Backdrop';
import { StatusPill } from '@/components/StatusPill';
import { Wordmark } from '@/components/Wordmark';
import {
  IconChevronLeft,
  IconChevronRight,
  IconGrid,
  IconLogout,
  IconMenu,
  IconSliders,
  IconStack,
} from '@/components/icons';
import { api } from '@/lib/api';
import { bootLocale } from '@/lib/boot';
import { cx } from '@/lib/cx';
import { formatUptime } from '@/lib/format';
import { t } from '@/lib/i18n';
import { bootstrapQuery, infoQuery } from '@/lib/queries';
import { useUi } from '@/stores/ui';

const NAV = [
  { to: '/', labelKey: 'nav.dashboards', index: '01', icon: IconGrid, exact: true },
  { to: '/resources', labelKey: 'nav.resources', index: '02', icon: IconStack, exact: false },
  { to: '/admin', labelKey: 'nav.admin', index: '03', icon: IconSliders, exact: false },
] as const;

function breadcrumb(pathname: string): string {
  if (pathname.startsWith('/resources')) return `${t('nav.sectionControl')} / ${t('nav.resources')}`;
  if (pathname.startsWith('/admin')) return `${t('nav.sectionSystem')} / ${t('nav.admin')}`;
  return `${t('nav.sectionControl')} / ${t('nav.dashboards')}`;
}

function initials(name: string): string {
  const parts = name.trim().split(/\s+/).filter(Boolean);
  if (parts.length === 0) return 'U';
  if (parts.length === 1) return parts[0]!.slice(0, 2).toUpperCase();
  return `${parts[0]![0] ?? ''}${parts[parts.length - 1]![0] ?? ''}`.toUpperCase();
}

export function AppShell() {
  const collapsed = useUi((state) => state.sidebarCollapsed);
  const toggleSidebar = useUi((state) => state.toggleSidebar);
  const mobileNavOpen = useUi((state) => state.mobileNavOpen);
  const setMobileNavOpen = useUi((state) => state.setMobileNavOpen);
  const location = useLocation();
  const navigate = useNavigate();
  const queryClient = useQueryClient();

  const bootstrap = useQuery(bootstrapQuery);
  const info = useQuery({ ...infoQuery, enabled: Boolean(bootstrap.data?.authenticated) });
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

  return (
    <div className="relative min-h-screen">
      <Backdrop />

      {mobileNavOpen && (
        <button
          type="button"
          aria-label={t('nav.menu')}
          onClick={() => setMobileNavOpen(false)}
          className="fixed inset-0 z-30 bg-black/70 backdrop-blur-sm lg:hidden"
        />
      )}

      <div className="relative z-10 flex min-h-screen">
        <aside
          style={{ width }}
          className={cx(
            'fixed inset-y-0 left-0 z-40 flex shrink-0 flex-col border-r border-line bg-ink-1/95 backdrop-blur transition-[transform,width] duration-300',
            'lg:sticky lg:top-0 lg:h-screen lg:translate-x-0',
            mobileNavOpen ? 'translate-x-0' : '-translate-x-full',
          )}
        >
          <div
            className={cx(
              'flex h-16 items-center border-b border-line',
              collapsed ? 'justify-center px-2' : 'justify-between px-4',
            )}
          >
            <Wordmark glyphOnly={collapsed} subtitle={t('app.tagline')} />
            <button
              type="button"
              onClick={toggleSidebar}
              className={cx(
                'btn btn-ghost hidden h-8 w-8 shrink-0 p-0 lg:inline-flex',
                collapsed && 'hidden',
              )}
              aria-label={t('nav.collapse')}
            >
              <IconChevronLeft width={15} height={15} />
            </button>
          </div>

          <nav className="flex-1 overflow-y-auto py-5" aria-label="Primary">
            <p className={cx('eyebrow mb-2 px-4', collapsed && 'sr-only')}>
              {t('nav.sectionControl')}
            </p>
            {NAV.slice(0, 2).map((item) => {
              const Icon = item.icon;
              return (
                <Link
                  key={item.to}
                  to={item.to}
                  activeOptions={{ exact: item.exact }}
                  activeProps={{ 'data-status': 'active' }}
                  className={cx('nav-item', collapsed && 'justify-center px-0')}
                  title={collapsed ? t(item.labelKey) : undefined}
                >
                  <Icon width={15} height={15} className="shrink-0 opacity-70" />
                  {!collapsed && <span className="flex-1 truncate">{t(item.labelKey)}</span>}
                  {!collapsed && <span className="nav-index text-right">{item.index}</span>}
                </Link>
              );
            })}

            <p className={cx('eyebrow mt-6 mb-2 px-4', collapsed && 'sr-only')}>
              {t('nav.sectionSystem')}
            </p>
            {NAV.slice(2).map((item) => {
              const Icon = item.icon;
              return (
                <Link
                  key={item.to}
                  to={item.to}
                  activeOptions={{ exact: item.exact }}
                  activeProps={{ 'data-status': 'active' }}
                  className={cx('nav-item', collapsed && 'justify-center px-0')}
                  title={collapsed ? t(item.labelKey) : undefined}
                >
                  <Icon width={15} height={15} className="shrink-0 opacity-70" />
                  {!collapsed && <span className="flex-1 truncate">{t(item.labelKey)}</span>}
                  {!collapsed && <span className="nav-index text-right">{item.index}</span>}
                </Link>
              );
            })}

            {!collapsed && (
              <div className="panel ticks mx-3 mt-8 p-3">
                <p className="eyebrow text-[8px]">{t('dashboard.schematic.title')}</p>
                <div className="mt-2 flex items-baseline justify-between">
                  <span className="mono text-[10px] text-paper-dim">{t('shell.connectors')}</span>
                  <span className="mono text-[13px] text-paper">00</span>
                </div>
                <div className="mt-1 flex items-baseline justify-between">
                  <span className="mono text-[10px] text-paper-dim">{t('shell.resources')}</span>
                  <span className="mono text-[13px] text-paper">00</span>
                </div>
                <div className="mt-2 border-t border-line-soft pt-2">
                  <span className="mono text-[9px] uppercase tracking-[0.14em] text-muted">
                    {t('shell.uptime')} · {formatUptime(info.data?.uptime_seconds ?? 0)}
                  </span>
                </div>
              </div>
            )}
          </nav>

          <div className="border-t border-line p-3">
            <div className={cx('flex items-center gap-3', collapsed && 'justify-center')}>
              <span className="grid h-9 w-9 shrink-0 place-items-center border border-line-strong bg-ink-2 text-[11px] tracking-[0.08em] text-paper">
                {initials(user?.display_name ?? '')}
              </span>
              {!collapsed && (
                <div className="min-w-0 flex-1">
                  <p className="truncate text-[13px] leading-tight text-paper">
                    {user?.display_name ?? '—'}
                  </p>
                  <p className="mono truncate text-[9px] uppercase tracking-[0.16em] text-muted">
                    {user?.role ?? ''}
                  </p>
                </div>
              )}
              {!collapsed && (
                <button
                  type="button"
                  onClick={() => logout.mutate()}
                  disabled={logout.isPending}
                  aria-label={t('nav.logout')}
                  title={t('nav.logout')}
                  className="btn btn-ghost h-8 w-8 p-0"
                >
                  <IconLogout width={15} height={15} />
                </button>
              )}
            </div>
            {collapsed && (
              <button
                type="button"
                onClick={toggleSidebar}
                className="btn btn-ghost mt-2 h-8 w-full p-0"
                aria-label={t('nav.expand')}
              >
                <IconChevronRight width={15} height={15} />
              </button>
            )}
            {!collapsed && (
              <p className="mono mt-3 flex items-center justify-between text-[8px] uppercase tracking-[0.16em] text-muted">
                <span>v{bootstrap.data?.version ?? '0.0.0'}</span>
                <span>{bootLocale()}</span>
              </p>
            )}
          </div>
        </aside>

        <div className="flex min-w-0 flex-1 flex-col">
          <header className="sticky top-0 z-20 flex h-16 items-center justify-between gap-4 border-b border-line bg-ink-0/80 px-4 backdrop-blur-md lg:px-8">
            <div className="flex min-w-0 items-center gap-3">
              <button
                type="button"
                onClick={() => setMobileNavOpen(true)}
                className="btn btn-ghost h-9 w-9 p-0 lg:hidden"
                aria-label={t('nav.menu')}
              >
                <IconMenu width={17} height={17} />
              </button>
              <p className="mono truncate text-[9px] uppercase tracking-[0.22em] text-muted">
                {breadcrumb(location.pathname)}
              </p>
            </div>
            <div className="flex items-center gap-2">
              <StatusPill />
              <span className="chip hidden md:inline-flex" title={t('shell.commandHint')}>
                <span className="text-muted">⌘K</span>
              </span>
            </div>
          </header>

          <main className="mx-auto w-full max-w-[1500px] flex-1 px-4 pb-16 pt-8 lg:px-8">
            <Outlet />
          </main>

          <footer className="border-t border-line px-4 py-4 lg:px-8">
            <p className="mono flex flex-wrap items-center gap-x-4 gap-y-1 text-[8px] uppercase tracking-[0.18em] text-muted">
              <span>
                {bootstrap.data?.instance_name ?? 'UniPanel'} · v{bootstrap.data?.version ?? '0.0.0'}
              </span>
              <span>M0 foundation</span>
              <span className="ml-auto">MIT · {bootLocale()}</span>
            </p>
          </footer>
        </div>
      </div>
    </div>
  );
}
