import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { RouterProvider } from '@tanstack/react-router';

import '@/styles/global.css';
import { useLocale } from '@/lib/i18n';
import { bootLocale } from '@/lib/boot';
import { router } from '@/router';

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 1,
      staleTime: 5 * 60 * 1000,
    },
  },
});

async function start() {
  const locale = bootLocale();
  await useLocale.getState().setLocale(locale);

  const rootElement = document.getElementById('root') as Element;
  const root = createRoot(rootElement);
  root.render(
    <StrictMode>
      <QueryClientProvider client={queryClient}>
        <RouterProvider router={router} />
      </QueryClientProvider>
    </StrictMode>,
  );
}

void start();
