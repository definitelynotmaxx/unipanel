import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { RouterProvider } from '@tanstack/react-router';

import '@/styles/global.css';
import { useLocale } from '@/lib/i18n';
import { bootLocale } from '@/lib/boot';
import { router } from '@/router';

async function start() {
  const locale = bootLocale();
  await useLocale.getState().setLocale(locale);

  const rootElement = document.getElementById('root') as Element;
  const root = createRoot(rootElement);
  root.render(
    <StrictMode>
      <RouterProvider router={router} />
    </StrictMode>,
  );
}

void start();