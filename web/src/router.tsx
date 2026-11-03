import { createRootRoute, createRoute, createRouter, Outlet } from '@tanstack/react-router';

import { AppShell } from '@/components/AppShell';
import { DashboardPage } from '@/routes/dashboard';
import { LoginPage } from '@/routes/login';
import { ResourcesPage } from '@/routes/resources';
import { AdminPage } from '@/routes/admin';
import { SetupPage } from '@/routes/setup';

const rootRoute = createRootRoute({
  component: () => <Outlet />,
});

const setupRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/setup',
  component: SetupPage,
});

const loginRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/login',
  component: LoginPage,
});

const appRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/',
  component: AppShell,
});

const dashboardRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/',
  component: DashboardPage,
});

const resourcesRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/resources',
  component: ResourcesPage,
});

const adminRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/admin',
  component: AdminPage,
});

const routeTree = rootRoute.addChildren([
  setupRoute,
  loginRoute,
  appRoute.addChildren([dashboardRoute, resourcesRoute, adminRoute]),
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