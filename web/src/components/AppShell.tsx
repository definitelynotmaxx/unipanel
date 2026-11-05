import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { Link, Outlet, useLocation, useNavigate } from '@tanstack/react-router';
import { useEffect } from 'react';

import { StatusPill } from '@/components/StatusPill';
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
import { bootLocale } from '@/lib/boot';
import { bootstrapQuery, infoQuery } from '@/lib/queries';
import { t } from '@/lib/i18n';
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
  const setMobileNavOpen = useUi((s) => s.setMobileNavOpen);
  const mobileNavOpen = useUi((s) => s.mobileNavOpen);
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

  const instanceName = info.data?.instance_name ?? bootstrap.data?.instance_name ?? '';

  return (
    <div className="flex h-screen overflow-hidden">
      {/* Mobile overlay */}
      {mobileNavOpen && (
        <div
          className="fixed inset-0 z-40 bg-black/40 lg:hidden"
          onClick={() => setMobileNavOpen(false)}
        />
      )}

      {/* ── Sidebar ── */}
      <aside
        className={
          'fixed inset-y-0 left-0 z-50 flex flex-col bg-white border-r border-[#e5e7eb] transition-all duration-300 ease-out' +
          (collapsed ? ' w-[64px]' : ' w-[220px]') +
          ' lg:sticky' +
          (mobileNavOpen ? ' translate-x-0' : ' -translate-x-full lg:translate-x-0')
        }
      >
        {/* Logo */}
        <div className="h-14 flex items-center justify-center border-b border-[#e5e7eb]">
          <span className="text-[10px] font-bold tracking-[0.2em] text-[#6b7280] uppercase">
            UniPanel
          </span>
        </div>

        {/* Navigation */}
        <nav className="flex-1 overflow-y-auto py-3">
          <div className="space-y-1 px-2">
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
                  className={
                    'flex items-center gap-3 h-10 px-2.5 rounded-md cursor-pointer transition-all duration-150' +
                    (active
                      ? ' bg-blue-50 text-blue-600 border-l-2 border-blue-500 pl-[9px]'
                      : ' text-gray-500 hover:bg-gray-50 hover:text-gray-700')
                  }
                  onClick={() => setMobileNavOpen(false)}
                  title={collapsed ? item.labelKey : undefined}
                >
                  <Icon className="h-[18px] w-[18px] shrink-0" />
                  {!collapsed && (
                    <span className="text-[13px] font-medium truncate">{t(item.labelKey)}</span>
                  )}
                </Link>
              );
            })}
          </div>
        </nav>

        {/* Collapse toggle */}
        <div className="border-t border-[#e5e7eb] py-2">
          <button
            type="button"
            onClick={toggleSidebar}
            className="flex items-center justify-center w-full h-10 text-gray-400 hover:text-gray-600 hover:bg-gray-50"
            title={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}
          >
            {collapsed ? (
              <IconChevronRight className="h-4 w-4" />
            ) : (
              <IconChevronLeft className="h-4 w-4" />
            )}
          </button>
        </div>

        {/* User bar */}
        {!collapsed && (
          <div className="border-t border-[#e5e7eb] p-3">
            <div className="flex items-center gap-2.5">
              <div className="w-8 h-8 rounded-md bg-gray-100 flex items-center justify-center text-[11px] font-semibold text-gray-600">
                {initials(user?.display_name ?? '')}
              </div>
              <div className="min-w-0 flex-1">
                <p className="text-[13px] font-medium text-gray-800 truncate">
                  {user?.display_name ?? '—'}
                </p>
                <p className="text-[10px] text-gray-400 font-mono uppercase tracking-wider">
                  {user?.role ?? ''}
                </p>
              </div>
              <button
                type="button"
                onClick={() => logout.mutate()}
                disabled={logout.isPending}
                className="text-gray-400 hover:text-red-500 transition-colors"
                title="Sign out"
              >
                <IconLogout className="h-4 w-4" />
              </button>
            </div>
          </div>
        )}
      </aside>

      {/* ── Main ── */}
      <div className="flex flex-1 flex-col min-w-0 overflow-hidden">
        {/* Top bar */}
        <header className="h-14 flex items-center justify-between px-4 lg:px-6 border-b border-[#e5e7eb] bg-white">
          <div className="flex items-center gap-3">
            <button
              type="button"
              onClick={() => setMobileNavOpen(true)}
              className="btn btn-icon btn-ghost lg:hidden"
              aria-label="Menu"
            >
              <IconMenu className="h-4 w-4" />
            </button>
            <p className="text-xs text-gray-400 font-mono tracking-wider uppercase">
              {instanceName}
            </p>
          </div>
          <StatusPill />
        </header>

        {/* Page content */}
        <main className="flex-1 overflow-y-auto p-4 lg:p-6">
          <Outlet />
        </main>

        {/* Footer */}
        <footer className="border-t border-[#e5e7eb] px-4 lg:px-6 py-2 bg-white">
          <p className="text-[9px] text-gray-400 font-mono tracking-wider">
            {bootstrap.data?.instance_name ?? 'UniPanel'} · v{bootstrap.data?.version ?? '—'} ·{' '}
            {bootLocale()} · MIT
          </p>
        </footer>
      </div>
    </div>
  );
}
