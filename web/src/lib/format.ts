/** Small formatting helpers shared by the shell and tables. */

/** Formats a duration in seconds as `12d 04h`, `04h 12m`, `12m 30s` or `30s`. */
export function formatUptime(seconds: number): string {
  const safe = Math.max(0, Math.floor(seconds));
  const days = Math.floor(safe / 86_400);
  const hours = Math.floor((safe % 86_400) / 3_600);
  const minutes = Math.floor((safe % 3_600) / 60);
  const secs = safe % 60;

  if (days > 0) return `${days}d ${pad(hours)}h`;
  if (hours > 0) return `${hours}h ${pad(minutes)}m`;
  if (minutes > 0) return `${minutes}m ${pad(secs)}s`;
  return `${secs}s`;
}

function pad(value: number): string {
  return value.toString().padStart(2, '0');
}

/** Formats an RFC 3339 timestamp for the admin table. */
export function formatTimestamp(value: string | undefined): string {
  if (!value) return '—';
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return new Intl.DateTimeFormat(undefined, {
    dateStyle: 'medium',
    timeStyle: 'short',
  }).format(date);
}

/** Turns `game_server` into `game server`. */
export function humanize(value: string): string {
  return value.replace(/_/g, ' ');
}
