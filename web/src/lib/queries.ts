/** Shared TanStack Query options. */
import { queryOptions } from '@tanstack/react-query';

import { api, queryKeys } from '@/lib/api';

export const bootstrapQuery = queryOptions({
  queryKey: queryKeys.bootstrap,
  queryFn: () => api.bootstrap(),
  staleTime: 15_000,
  retry: 1,
});

export const infoQuery = queryOptions({
  queryKey: queryKeys.info,
  queryFn: () => api.info(),
  staleTime: 60_000,
  retry: 1,
});

export const healthQuery = queryOptions({
  queryKey: queryKeys.health,
  queryFn: () => api.health(),
  refetchInterval: 30_000,
  retry: 1,
  staleTime: 10_000,
});
