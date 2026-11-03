import { t } from '@/lib/i18n';

const MONO = 'var(--font-mono)';
const DISPLAY = 'var(--font-display)';

interface SchematicProps {
  /** Hides the outer frame and secondary labels on narrow layouts. */
  compact?: boolean;
  connectors?: number;
  resources?: number;
}

/**
 * Hand-drawn signal-path diagram.
 *
 * It documents the architecture (browser ⇄ panel ⇄ upstream APIs) instead of
 * showing fake data, which keeps the empty dashboard honest and still gives the
 * page a centre of gravity.
 */
export function Schematic({ compact = false, connectors = 0, resources = 0 }: SchematicProps) {
  const upstreams = [
    { label: 'PTERODACTYL', sub: 'CLIENT API', y: 78 },
    { label: 'DOCKER', sub: 'ENGINE API', y: 144 },
    { label: 'PROXMOX', sub: 'PVE API', y: 210 },
  ];

  return (
    <svg
      viewBox="0 0 560 380"
      role="img"
      aria-label={`${t('dashboard.schematic.title')}: ${t('dashboard.schematic.browser')} → ${t('dashboard.schematic.core')} → ${t('dashboard.schematic.upstream')}`}
      className="h-auto w-full"
    >
      <defs>
        <marker
          id="schematic-arrow"
          viewBox="0 0 8 8"
          refX="7"
          refY="4"
          markerWidth="7"
          markerHeight="7"
          orient="auto-start-reverse"
        >
          <path d="M0 0 8 4 0 8Z" fill="var(--color-line-strong)" />
        </marker>
      </defs>

      {!compact && (
        <>
          <rect
            x="0.5"
            y="0.5"
            width="559"
            height="379"
            fill="none"
            stroke="var(--color-line)"
          />
          <path d="M0 12V0h12M548 0h12v12M560 368v12h-12M12 380H0v-12" stroke="var(--color-line-strong)" />
          <text x="24" y="30" fontFamily={MONO} fontSize="8" letterSpacing="2.2" fill="var(--color-muted)">
            SIGNAL TOPOLOGY
          </text>
          <text x="536" y="30" textAnchor="end" fontFamily={MONO} fontSize="8" letterSpacing="2.2" fill="var(--color-line-strong)">
            01 / 03
          </text>
        </>
      )}

      {/* Upstream systems */}
      {upstreams.map((node) => (
        <g key={node.label}>
          <rect
            x="24"
            y={node.y}
            width="118"
            height="38"
            fill="var(--color-ink-2)"
            stroke="var(--color-line)"
          />
          <rect x="24" y={node.y} width="3" height="38" fill="var(--color-line-strong)" />
          <text
            x="38"
            y={node.y + 16}
            fontFamily={MONO}
            fontSize="8.5"
            letterSpacing="1.1"
            fill="var(--color-paper-dim)"
          >
            {node.label}
          </text>
          <text
            x="38"
            y={node.y + 28}
            fontFamily={MONO}
            fontSize="7"
            letterSpacing="1.4"
            fill="var(--color-muted)"
          >
            {node.sub}
          </text>
          <path
            d={`M142 ${node.y + 19} H178 V169 H214`}
            fill="none"
            stroke="var(--color-line-strong)"
            strokeWidth="1"
          />
          <path
            d={`M142 ${node.y + 19} H178 V169 H214`}
            fill="none"
            stroke="var(--color-sodium)"
            strokeOpacity="0.5"
            strokeWidth="1"
            className="dash-flow"
          />
          <circle cx="142" cy={node.y + 19} r="2" fill="var(--color-line-strong)" />
        </g>
      ))}

      {/* Core */}
      <rect x="214" y="110" width="132" height="118" fill="var(--color-ink-1)" stroke="var(--color-line-strong)" />
      <rect x="220" y="116" width="120" height="106" fill="none" stroke="var(--color-line-soft)" />
      <rect x="214" y="110" width="132" height="2" fill="var(--color-sodium)" />
      <text
        x="280"
        y="163"
        textAnchor="middle"
        fontFamily={DISPLAY}
        fontSize="17"
        letterSpacing="1.5"
        fill="var(--color-paper)"
      >
        UNIPANEL
      </text>
      <text
        x="280"
        y="180"
        textAnchor="middle"
        fontFamily={MONO}
        fontSize="7"
        letterSpacing="1.6"
        fill="var(--color-muted)"
      >
        POLL · CACHE · SSE
      </text>
      <line x1="244" y1="196" x2="316" y2="196" stroke="var(--color-line)" />
      <rect x="249" y="203" width="5" height="7" fill="var(--color-sodium)" className="cursor" />
      <text x="259" y="210" fontFamily={MONO} fontSize="7" letterSpacing="1.2" fill="var(--color-paper-dim)">
        READY
      </text>
      <text x="303" y="210" fontFamily={MONO} fontSize="7" letterSpacing="1.2" fill="var(--color-muted)">
        0 CLIENTS
      </text>

      {/* Browser */}
      <rect x="418" y="132" width="118" height="74" fill="var(--color-ink-2)" stroke="var(--color-line)" />
      <path d="M418 132h118" stroke="var(--color-line)" />
      <circle cx="428" cy="140" r="1.6" fill="var(--color-line-strong)" />
      <circle cx="434" cy="140" r="1.6" fill="var(--color-line-strong)" />
      <text
        x="477"
        y="171"
        textAnchor="middle"
        fontFamily={MONO}
        fontSize="8.5"
        letterSpacing="1.4"
        fill="var(--color-paper-dim)"
      >
        BROWSER
      </text>
      <text
        x="477"
        y="185"
        textAnchor="middle"
        fontFamily={MONO}
        fontSize="7"
        letterSpacing="1.4"
        fill="var(--color-muted)"
      >
        REACT SPA
      </text>

      <path
        d="M346 169 H418"
        fill="none"
        stroke="var(--color-line-strong)"
        markerEnd="url(#schematic-arrow)"
      />
      <text x="356" y="161" fontFamily={MONO} fontSize="7" letterSpacing="1.4" fill="var(--color-muted)">
        SSE
      </text>

      <text x="24" y="352" fontFamily={MONO} fontSize="8" letterSpacing="1.6" fill="var(--color-paper-dim)">
        {connectors} CONNECTORS
      </text>
      <circle cx="134" cy="349" r="1.5" fill="var(--color-line-strong)" />
      <text x="146" y="352" fontFamily={MONO} fontSize="8" letterSpacing="1.6" fill="var(--color-paper-dim)">
        {resources} RESOURCES
      </text>
      <text x="536" y="352" textAnchor="end" fontFamily={MONO} fontSize="8" letterSpacing="1.6" fill="var(--color-muted)">
        HTTPS ONLY
      </text>
    </svg>
  );
}
