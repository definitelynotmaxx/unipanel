import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useState } from 'react';
import { AlertCircle, CheckCircle2, Loader2, TestTube2, Trash2, X } from 'lucide-react';

import { IconCode, IconServer } from '@/components/icons';
import { api } from '@/lib/api';

// ── Types ──

export interface Connector {
  id: string;
  name: string;
  kind: string;
  api_url: string;
  is_active: boolean;
  created_at: string;
}

interface ConnectorRequest {
  name: string;
  kind: string;
  api_url: string;
  api_key: string;
  is_active?: boolean;
}

interface ConnectorTestRequest {
  kind: string;
  api_url: string;
  api_key: string;
}

interface ConnectorTestResponse {
  ok: boolean;
  latency_ms?: number;
  message: string;
  details: Record<string, unknown>;
}

// ── Connector kind config ──

const CONNECTOR_KINDS = [
  { value: 'Pterodactyl', label: 'Pterodactyl', color: '#8b5cf6', bg: 'bg-violet-50', text: 'text-violet-700', border: 'border-violet-200' },
  { value: 'Docker', label: 'Docker', color: '#3b82f6', bg: 'bg-blue-50', text: 'text-blue-700', border: 'border-blue-200' },
  { value: 'Proxmox', label: 'Proxmox', color: '#f97316', bg: 'bg-orange-50', text: 'text-orange-700', border: 'border-orange-200' },
  { value: 'REST_API', label: 'REST API', color: '#6b7280', bg: 'bg-gray-50', text: 'text-gray-700', border: 'border-gray-200' },
] as const;

function getKindConfig(kind: string) {
  return CONNECTOR_KINDS.find((k) => k.value === kind) ?? CONNECTOR_KINDS[3];
}

// ── API helpers ──

async function listConnectors(): Promise<Connector[]> {
  return api.get<Connector[]>('/connectors');
}

async function createConnector(payload: ConnectorRequest): Promise<Connector> {
  return api.post<Connector>('/connectors', payload);
}

async function deleteConnector(id: string): Promise<void> {
  return api.delete<void>(`/connectors/${id}`);
}

async function testConnector(payload: ConnectorTestRequest): Promise<ConnectorTestResponse> {
  return api.post<ConnectorTestResponse>('/connectors/test', payload);
}

// ── Query keys ──

const connectorQueryKeys = {
  all: ['connectors'] as const,
  list: ['connectors', 'list'] as const,
};

// ── Main Page ──

export function ConnectorPage() {
  const [showForm, setShowForm] = useState(false);
  const [testResult, setTestResult] = useState<{ ok: boolean; message: string; latency_ms?: number } | null>(null);
  const [testError, setTestError] = useState<string | null>(null);
  const [deletingId, setDeletingId] = useState<string | null>(null);
  const [deleteError, setDeleteError] = useState<string | null>(null);
  const queryClient = useQueryClient();

  const { data: connectors = [], isLoading, error } = useQuery({
    queryKey: connectorQueryKeys.list,
    queryFn: listConnectors,
  });

  const createMutation = useMutation({
    mutationFn: createConnector,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: connectorQueryKeys.list });
      setShowForm(false);
      setTestResult(null);
      setTestError(null);
    },
  });

  const deleteMutation = useMutation({
    mutationFn: deleteConnector,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: connectorQueryKeys.list });
      setDeleteError(null);
    },
  });

  const testMutation = useMutation({
    mutationFn: testConnector,
  });

  const handleDelete = async (id: string) => {
    if (!window.confirm('Delete this connector? This action cannot be undone.')) return;
    setDeletingId(id);
    setDeleteError(null);
    try {
      await deleteMutation.mutateAsync(id);
    } catch (err: unknown) {
      setDeleteError(err instanceof Error ? err.message : 'Failed to delete connector');
    } finally {
      setDeletingId(null);
    }
  };

  return (
    <div className="space-y-6 animate-in">
      {/* Header */}
      <div className="flex items-start justify-between gap-4">
        <div>
          <h1 className="heading-lg">Connectors</h1>
          <p className="text-sm text-gray-500 mt-1">
            Manage external panel integrations
          </p>
        </div>
        <button
          type="button"
          className="btn btn-primary"
          onClick={() => {
            setShowForm(true);
            setTestResult(null);
            setTestError(null);
          }}
        >
          <PlusIcon className="w-4 h-4" />
          Add connector
        </button>
      </div>

      {/* Error alert from query */}
      {error && (
        <div className="alert-danger" role="alert">
          <AlertCircle className="h-4 w-4 shrink-0 mt-0.5" />
          <span>Failed to load connectors. Please try again.</span>
        </div>
      )}

      {/* Delete error alert */}
      {deleteError && (
        <div className="alert-danger" role="alert">
          <AlertCircle className="h-4 w-4 shrink-0 mt-0.5" />
          <span>{deleteError}</span>
        </div>
      )}

      {/* Loading state */}
      {isLoading ? (
        <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
          {Array.from({ length: 3 }).map((_, i) => (
            <div key={i} className="card p-5">
              <div className="flex items-center gap-3 mb-4">
                <div className="w-10 h-10 rounded-lg bg-gray-100 skeleton" />
                <div className="flex-1">
                  <div className="h-4 w-24 bg-gray-100 skeleton rounded" />
                  <div className="h-3 w-16 bg-gray-100 skeleton rounded mt-1.5" />
                </div>
              </div>
              <div className="h-3 w-full bg-gray-100 skeleton rounded" />
            </div>
          ))}
        </div>
      ) : connectors.length === 0 ? (
        /* Empty state */
        <div className="card p-6">
          <div className="empty-state">
            <div className="empty-state-icon">
              <PlugIcon className="w-5 h-5 text-gray-400" />
            </div>
            <p className="empty-state-title">No connectors configured yet</p>
            <p className="empty-state-desc">
              Add your first connector to manage external panels. Support Pterodactyl, Docker, Proxmox, and any REST API.
            </p>
          </div>
        </div>
      ) : (
        /* Connector cards grid */
        <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
          {connectors.map((connector, index) => (
            <ConnectorCard
              key={connector.id}
              connector={connector}
              onDelete={handleDelete}
              isDeleting={deletingId === connector.id}
              index={index}
            />
          ))}
        </div>
      )}

      {/* Add connector modal */}
      {showForm && (
        <ConnectorFormModal
          onClose={() => {
            setShowForm(false);
            setTestResult(null);
            setTestError(null);
          }}
          onSubmit={(payload) => createMutation.mutate(payload)}
          isPending={createMutation.isPending}
          testResult={testResult}
          testError={testError}
          onTest={(payload) => {
            setTestResult(null);
            setTestError(null);
            testMutation.mutate(payload, {
              onSuccess: (result) => {
                setTestResult({
                  ok: result.ok,
                  message: result.message,
                  latency_ms: result.latency_ms,
                });
              },
              onError: (err: unknown) => {
                setTestError(err instanceof Error ? err.message : 'Test failed');
              },
            });
          }}
          isTesting={testMutation.isPending}
        />
      )}
    </div>
  );
}

// ── Connector Card ──

function ConnectorCard({
  connector,
  onDelete,
  isDeleting,
  index,
}: {
  connector: Connector;
  onDelete: (id: string) => void;
  isDeleting: boolean;
  index: number;
}) {
  const kindConfig = getKindConfig(connector.kind);

  return (
    <div
      className={`card p-5 animate-in animate-in-delay-${Math.min(index, 4)}`}
    >
      {/* Card header */}
      <div className="flex items-start justify-between gap-3 mb-4">
        <div className="flex items-center gap-3 min-w-0 flex-1">
          <div
            className="w-10 h-10 rounded-lg flex items-center justify-center shrink-0"
            style={{ backgroundColor: `${kindConfig.color}15` }}
          >
            <KindIcon kind={connector.kind} color={kindConfig.color} />
          </div>
          <div className="min-w-0 flex-1">
            <h3 className="text-sm font-semibold text-gray-900 truncate">
              {connector.name}
            </h3>
            <span
              className={`inline-flex items-center px-2 py-0.5 rounded-md text-[10px] font-semibold uppercase tracking-wider border ${kindConfig.bg} ${kindConfig.text} ${kindConfig.border}`}
            >
              {connector.kind}
            </span>
          </div>
        </div>
        <span
          className={`inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-semibold uppercase tracking-wider shrink-0 ${
            connector.is_active
              ? 'bg-green-50 text-green-700 border border-green-200'
              : 'bg-gray-100 text-gray-500 border border-gray-200'
          }`}
        >
          <span className={`w-1.5 h-1.5 rounded-full ${
            connector.is_active ? 'bg-green-500' : 'bg-gray-400'
          }`} />
          {connector.is_active ? 'Active' : 'Inactive'}
        </span>
      </div>

      {/* API URL */}
      <div className="mb-4">
        <p className="text-xs text-gray-500 mb-1">API URL</p>
        <p className="text-xs font-mono text-gray-700 truncate bg-gray-50 rounded px-2.5 py-1.5">
          {connector.api_url}
        </p>
      </div>

      {/* Created at */}
      <p className="text-[10px] text-gray-400 font-mono mb-4">
        Created {new Date(connector.created_at).toLocaleDateString()}
      </p>

      {/* Actions */}
      <div className="flex items-center gap-2 pt-3 border-t border-gray-100">
        <button
          type="button"
          className="btn btn-sm flex-1"
          onClick={() => onDelete(connector.id)}
          disabled={isDeleting}
        >
          {isDeleting ? (
            <Loader2 className="w-3.5 h-3.5 animate-spin" />
          ) : (
            <Trash2 className="w-3.5 h-3.5" />
          )}
          Delete
        </button>
      </div>
    </div>
  );
}

// ── Modal Form ──

function ConnectorFormModal({
  onClose,
  onSubmit,
  isPending,
  testResult,
  testError,
  onTest,
  isTesting,
}: {
  onClose: () => void;
  onSubmit: (payload: ConnectorRequest) => void;
  isPending: boolean;
  testResult: { ok: boolean; message: string; latency_ms?: number } | null;
  testError: string | null;
  onTest: (payload: ConnectorTestRequest) => void;
  isTesting: boolean;
}) {
  const [name, setName] = useState('');
  const [kind, setKind] = useState('Pterodactyl');
  const [apiUrl, setApiUrl] = useState('');
  const [apiKey, setApiKey] = useState('');
  const [urlError, setUrlError] = useState('');
  const [nameError, setNameError] = useState('');
  const [apiKeyError, setApiKeyError] = useState('');

  const validateUrl = (url: string): boolean => {
    if (!url.trim()) {
      setUrlError('API URL is required');
      return false;
    }
    try {
      new URL(url);
      setUrlError('');
      return true;
    } catch {
      setUrlError('Please enter a valid URL (e.g., https://panel.example.com)');
      return false;
    }
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    let valid = true;

    if (!name.trim()) {
      setNameError('Name is required');
      valid = false;
    } else {
      setNameError('');
    }

    if (!validateUrl(apiUrl)) {
      valid = false;
    }

    if (!apiKey.trim()) {
      setApiKeyError('API key is required');
      valid = false;
    } else {
      setApiKeyError('');
      valid = false;
    }

    if (!valid) return;

    onSubmit({
      name: name.trim(),
      kind,
      api_url: apiUrl.trim(),
      api_key: apiKey,
    });
  };

  const handleTest = () => {
    if (!validateUrl(apiUrl)) return;
    if (!name.trim()) {
      setNameError('Name is required');
      return;
    }
    if (!apiKey.trim()) {
      setApiKeyError('API key is required');
      return;
    }
    onTest({ kind, api_url: apiUrl.trim(), api_key: apiKey });
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
      {/* Backdrop */}
      <div
        className="absolute inset-0 bg-black/40 backdrop-blur-sm"
        onClick={onClose}
      />

      {/* Modal */}
      <div className="relative bg-white rounded-xl border border-gray-200 shadow-xl w-full max-w-lg animate-in">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-gray-100">
          <h2 className="text-lg font-semibold text-gray-900">Add connector</h2>
          <button
            type="button"
            className="btn btn-ghost btn-icon"
            onClick={onClose}
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Form */}
        <form onSubmit={handleSubmit} className="p-6 space-y-5">
          {/* Name */}
          <div className="form-group">
            <label htmlFor="connector-name" className="form-label">
              Name <span className="text-red-500">*</span>
            </label>
            <input
              id="connector-name"
              type="text"
              className={`input ${nameError ? 'input-error' : ''}`}
              placeholder="My Pterodactyl Panel"
              value={name}
              onChange={(e) => { setName(e.target.value); setNameError(''); }}
              aria-invalid={nameError ? 'true' : undefined}
            />
            {nameError && <p className="form-error">{nameError}</p>}
          </div>

          {/* Kind */}
          <div className="form-group">
            <label htmlFor="connector-kind" className="form-label">
              Type <span className="text-red-500">*</span>
            </label>
            <select
              id="connector-kind"
              className="input"
              value={kind}
              onChange={(e) => setKind(e.target.value)}
            >
              {CONNECTOR_KINDS.map((k) => (
                <option key={k.value} value={k.value}>{k.label}</option>
              ))}
            </select>
          </div>

          {/* API URL */}
          <div className="form-group">
            <label htmlFor="connector-url" className="form-label">
              API URL <span className="text-red-500">*</span>
            </label>
            <input
              id="connector-url"
              type="url"
              className={`input ${urlError ? 'input-error' : ''}`}
              placeholder="https://panel.example.com"
              value={apiUrl}
              onChange={(e) => { setApiUrl(e.target.value); setUrlError(''); }}
              onBlur={(e) => validateUrl(e.target.value)}
              aria-invalid={urlError ? 'true' : undefined}
            />
            {urlError && <p className="form-error">{urlError}</p>}
          </div>

          {/* API Key */}
          <div className="form-group">
            <label htmlFor="connector-key" className="form-label">
              API Key <span className="text-red-500">*</span>
            </label>
            <input
              id="connector-key"
              type="password"
              className={`input ${apiKeyError ? 'input-error' : ''}`}
              placeholder="Enter your API key"
              value={apiKey}
              onChange={(e) => { setApiKey(e.target.value); setApiKeyError(''); }}
              aria-invalid={apiKeyError ? 'true' : undefined}
            />
            {apiKeyError && <p className="form-error">{apiKeyError}</p>}
          </div>

          {/* Test result */}
          {(testResult || testError) && (
            <div className={`flex items-start gap-2.5 p-3 rounded-lg text-sm ${
              testError
                ? 'bg-red-50 text-red-700 border border-red-200'
                : testResult?.ok
                  ? 'bg-green-50 text-green-700 border border-green-200'
                  : 'bg-amber-50 text-amber-700 border border-amber-200'
            }`}>
              {testResult?.ok ? (
                <CheckCircle2 className="w-4 h-4 shrink-0 mt-0.5" />
              ) : testError ? (
                <AlertCircle className="w-4 h-4 shrink-0 mt-0.5" />
              ) : (
                <AlertCircle className="w-4 h-4 shrink-0 mt-0.5" />
              )}
              <div>
                <p>{testError ?? testResult?.message}</p>
                {testResult?.latency_ms != null && (
                  <p className="text-xs mt-1 opacity-75">
                    Response time: {testResult.latency_ms}ms
                  </p>
                )}
              </div>
            </div>
          )}

          {/* Buttons */}
          <div className="flex items-center gap-3 pt-2">
            <button
              type="button"
              className="btn flex-1"
              onClick={handleTest}
              disabled={isTesting || !apiUrl.trim() || !apiKey.trim()}
            >
              {isTesting ? (
                <Loader2 className="w-4 h-4 animate-spin" />
              ) : (
                <TestTube2 className="w-4 h-4" />
              )}
              Test connection
            </button>
          </div>

          <div className="flex items-center gap-3 pt-2 border-t border-gray-100">
            <button
              type="submit"
              className="btn btn-primary flex-1"
              disabled={isPending}
            >
              {isPending ? (
                <Loader2 className="w-4 h-4 animate-spin" />
              ) : null}
              Create connector
            </button>
            <button
              type="button"
              className="btn flex-1"
              onClick={onClose}
              disabled={isPending}
            >
              Cancel
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

// ── Icon Components ──

function KindIcon({ kind, color }: { kind: string; color: string }) {
  switch (kind) {
    case 'Pterodactyl':
      return <IconBox className="w-5 h-5" style={{ color }} />;
    case 'Docker':
      return <IconDocker className="w-5 h-5" style={{ color }} />;
    case 'Proxmox':
      return <IconServer className="w-5 h-5" style={{ color }} />;
    case 'REST_API':
    default:
      return <IconCode className="w-5 h-5" style={{ color }} />;
  }
}

function PlusIcon(props: React.SVGProps<SVGSVGElement>) {
  return (
    <svg {...props} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
      <path d="M5 12h14" />
      <path d="M12 5v14" />
    </svg>
  );
}

function PlugIcon(props: React.SVGProps<SVGSVGElement>) {
  return (
    <svg {...props} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
      <path d="M12 22v-5" />
      <path d="M9 8V2" />
      <path d="M15 8V2" />
      <path d="M18 8V5a2 2 0 0 0-2-2H8a2 2 0 0 0-2 2v3" />
      <path d="M12 13v4" />
      <path d="M8 21h8" />
    </svg>
  );
}

function IconBox(props: React.SVGProps<SVGSVGElement>) {
  return (
    <svg {...props} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
      <path d="M21 8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16Z" />
      <path d="m3.3 7 8.7 5 8.7-5" />
      <path d="M12 22V12" />
    </svg>
  );
}

function IconDocker(props: React.SVGProps<SVGSVGElement>) {
  return (
    <svg {...props} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
      <rect x="2" y="10" width="20" height="10" rx="2" />
      <path d="M6 10V6a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v4" />
      <path d="M14 10V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v6" />
      <line x1="6" y1="15" x2="6.01" y2="15" />
      <line x1="10" y1="15" x2="10.01" y2="15" />
    </svg>
  );
}
