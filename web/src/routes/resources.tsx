import { useNavigate } from '@tanstack/react-router';
import { useQuery } from '@tanstack/react-query';

import { bootstrapQuery } from '@/lib/queries';

export function ResourcesPage() {
  const navigate = useNavigate();
  const bootstrap = useQuery(bootstrapQuery);
  const authenticated = bootstrap.data?.authenticated ?? false;

  return (
    <div className="space-y-5 animate-in">
      <div>
        <h1 className="heading-lg">All Resources</h1>
        <p className="text-sm text-gray-500 mt-1">
          Servers, containers, and virtual machines
        </p>
      </div>

      <div className="card p-6">
        {authenticated ? (
          <div className="empty-state">
            <div className="w-12 h-12 rounded-xl bg-gray-100 flex items-center justify-center text-gray-400">
              <span className="text-2xl">🖥️</span>
            </div>
            <p className="text-sm font-semibold text-gray-700">Nothing to list</p>
            <p className="text-xs text-gray-400 max-w-[340px]">
              Resources appear as soon as a connector reports them. Actions, consoles and files follow the capability flags of each connector.
            </p>
            <button className="btn btn-primary btn-sm mt-2" onClick={() => navigate({ to: '/admin' })}>
              Add connector
            </button>
          </div>
        ) : (
          <div className="empty-state">
            <div className="w-12 h-12 rounded-xl bg-gray-100 flex items-center justify-center text-gray-400">
              <span className="text-2xl">🔌</span>
            </div>
            <p className="text-sm font-semibold text-gray-700">No connectors configured</p>
            <p className="text-xs text-gray-400 max-w-[340px]">
              Connect to Docker, Pterodactyl, Proxmox or any REST API to see server inventory here.
            </p>
          </div>
        )}
      </div>
    </div>
  );
}
