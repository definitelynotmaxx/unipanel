import { createRootRoute, createRoute, createRouter, Outlet } from '@tanstack/react-router';

import { AppShell } from '@/components/AppShell';
import { AdminPage } from '@/routes/admin';
import { ConnectorPage } from '@/routes/connectors';
import { DashboardPage } from '@/routes/dashboard';
import { LoginPage } from '@/routes/login';
import { ResourcesPage } from '@/routes/resources';
import { SetupPage } from '@/routes/setup';

const rootRoute = createRootRoute({
  component: () => <Outlet />,
});

const setupRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: 'setup', // relative path; TanStack prepends /
  component: SetupPage,
});

const loginRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: 'login',
  component: LoginPage,
});

// Pathless layout wrapper that draws the sidebar shell.
const appRoute = createRoute({
  getParentRoute: () => rootRoute,
  id: 'app-layout',
  component: AppShell,
});

const dashboardRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/',
  component: DashboardPage,
});

const resourcesRoute = createRoute({
  getParentRoute: () => appRoute,
  path: 'resources',
  component: ResourcesPage,
});

const adminRoute = createRoute({
  getParentRoute: () => appRoute,
  path: 'admin',
  component: AdminPage,
});

const connectorsRoute = createRoute({
  getParentRoute: () => appRoute,
  path: 'connectors',
  component: ConnectorPage,
});

const routeTree = rootRoute.addChildren([
  setupRoute,
  loginRoute,
  appRoute.addChildren([dashboardRoute, resourcesRoute, connectorsRoute, adminRoute]),
]);

export const router = createRouter({
  routeTree,
  defaultPreload: 'intent',
});

declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router;
  }
}