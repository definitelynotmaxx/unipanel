/**
 * Typed API client.
 *
 * Types come from `schema.d.ts`, generated from the server's OpenAPI document, so
 * the client cannot drift from the Rust DTOs. The session cookie is `HttpOnly`;
 * the CSRF token lives in memory and is refreshed by every bootstrap/login call.
 */
import type { components } from '@/api/schema';

export type BootstrapResponse = components['schemas']['BootstrapResponse'];
export type SetupRequest = components['schemas']['SetupRequest'];
export type LoginRequest = components['schemas']['LoginRequest'];
export type SessionResponse = components['schemas']['SessionResponse'];
export type UserDto = components['schemas']['UserDto'];
export type SystemInfoResponse = components['schemas']['SystemInfoResponse'];
export type HealthResponse = components['schemas']['HealthResponse'];
export type ApiErrorBody = components['schemas']['ApiErrorBody'];

let csrfToken = '';

export function setCsrfToken(token: string): void {
  csrfToken = token;
}

export function getCsrfToken(): string {
  return csrfToken;
}

/** An error returned by the API, carrying a translation key. */
export class ApiError extends Error {
  readonly status: number;
  readonly key: string;
  readonly params: Record<string, unknown>;
  readonly requestId?: string;

  constructor(
    status: number,
    key: string,
    params: Record<string, unknown> = {},
    requestId?: string,
  ) {
    super(key);
    this.name = 'ApiError';
    this.status = status;
    this.key = key;
    this.params = params;
    this.requestId = requestId;
  }
}

interface RequestOptions {
  method?: 'GET' | 'POST';
  body?: unknown;
  signal?: AbortSignal;
}

async function request<T>(path: string, options: RequestOptions = {}): Promise<T> {
  const method = options.method ?? 'GET';
  const headers: Record<string, string> = { Accept: 'application/json' };
  if (options.body !== undefined) headers['Content-Type'] = 'application/json';
  if (method !== 'GET' && csrfToken) headers['X-CSRF-Token'] = csrfToken;

  let response: Response;
  try {
    response = await fetch(`/api/v1${path}`, {
      method,
      credentials: 'same-origin',
      headers,
      body: options.body === undefined ? undefined : JSON.stringify(options.body),
      signal: options.signal,
    });
  } catch (cause) {
    if (cause instanceof DOMException && cause.name === 'AbortError') throw cause;
    throw new ApiError(0, 'internal');
  }

  if (response.status === 204) return undefined as T;

  const text = await response.text();
  let payload: unknown;
  if (text) {
    try {
      payload = JSON.parse(text);
    } catch {
      payload = undefined;
    }
  }

  if (!response.ok) {
    const body = payload as Partial<ApiErrorBody> | undefined;
    throw new ApiError(
      response.status,
      body?.error?.key ?? 'internal',
      (body?.error?.params ?? {}) as Record<string, unknown>,
      body?.request_id ?? undefined,
    );
  }

  return payload as T;
}

export const api = {
  async bootstrap(): Promise<BootstrapResponse> {
    const body = await request<BootstrapResponse>('/bootstrap');
    setCsrfToken(body.csrf_token);
    return body;
  },

  async setup(payload: SetupRequest): Promise<SessionResponse> {
    const body = await request<SessionResponse>('/setup', { method: 'POST', body: payload });
    setCsrfToken(body.csrf_token);
    return body;
  },

  async login(payload: LoginRequest): Promise<SessionResponse> {
    const body = await request<SessionResponse>('/auth/login', { method: 'POST', body: payload });
    setCsrfToken(body.csrf_token);
    return body;
  },

  async logout(): Promise<void> {
    await request<void>('/auth/logout', { method: 'POST', body: {} });
    setCsrfToken('');
  },

  session(): Promise<SessionResponse> {
    return request<SessionResponse>('/auth/session');
  },

  info(): Promise<SystemInfoResponse> {
    return request<SystemInfoResponse>('/system/info');
  },

  health(): Promise<HealthResponse> {
    return request<HealthResponse>('/system/health');
  },

  /** Generic GET helper for ad-hoc endpoints. */
  get<T>(path: string): Promise<T> {
    return request<T>(path);
  },

  /** Generic POST helper for ad-hoc endpoints. */
  post<T>(path: string, body: unknown): Promise<T> {
    return request<T>(path, { method: 'POST', body });
  },

  /** Generic DELETE helper for ad-hoc endpoints. */
  delete<T>(path: string): Promise<T> {
    return request<T>(path, { method: 'POST', body: {} });
  },
};

/** Query keys in one place so invalidation stays consistent. */
export const queryKeys = {
  bootstrap: ['bootstrap'] as const,
  info: ['system', 'info'] as const,
  health: ['system', 'health'] as const,
};
