import { Badge } from "../../components/ui/Badge";
import { ActionButton } from "../../components/ui/ActionButton";
import { InfoTooltip } from "../../components/ui/InfoTooltip";
import type { DriverIssue } from "../../lib/types";
import {
  severityVariant,
  severityLabel,
  kindLabel,
  kindHelpId,
  canUninstall,
  canRemovePhantom,
} from "./DriverAuditHelpers";

interface Props {
  issue: DriverIssue;
  recentlyChanged: boolean;
  uninstallBusy: string | null;
  removePhantomBusy: string | null;
  onUninstall: (issue: DriverIssue) => void;
  onRemovePhantom: (issue: DriverIssue) => void;
}

export function DriverIssueCard({
  issue,
  recentlyChanged,
  uninstallBusy,
  removePhantomBusy,
  onUninstall,
  onRemovePhantom,
}: Props) {
  return (
    <div
      data-recently-changed={recentlyChanged ? "true" : undefined}
      className="bg-bg-card border border-border rounded-lg p-5 theme-card transition-all duration-700 data-[recently-changed=true]:ring-2 data-[recently-changed=true]:ring-success/40"
    >
      <div className="flex items-start justify-between gap-4">
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 mb-2 flex-wrap">
            <p className="text-sm font-medium text-fg-primary break-words">{issue.deviceName}</p>
            <Badge variant={severityVariant(issue.severity)}>
              {severityLabel(issue.severity)}
            </Badge>
            <span className="inline-flex items-center gap-1">
              <Badge variant="neutral">{kindLabel(issue.kind.kind)}</Badge>
              <InfoTooltip textId={kindHelpId(issue.kind.kind)} />
            </span>
          </div>
          <p className="text-xs text-fg-muted mb-1">{issue.reason}</p>
          <div className="flex flex-wrap gap-x-4 gap-y-0.5 text-xs text-fg-dim">
            {issue.manufacturer && <span>{issue.manufacturer}</span>}
            {issue.driverVersion && <span>v{issue.driverVersion}</span>}
            {issue.driverDate && <span>{issue.driverDate}</span>}
            {issue.infName && <span className="font-mono">{issue.infName}</span>}
          </div>
        </div>
        <div className="flex gap-2 shrink-0">
          {canUninstall(issue) && (
            <ActionButton
              variant="danger"
              requiresAdmin
              loading={uninstallBusy === issue.id}
              onClick={() => onUninstall(issue)}
            >
              Désinstaller
            </ActionButton>
          )}
          {canRemovePhantom(issue) && (
            <ActionButton
              variant="warning"
              requiresAdmin
              loading={removePhantomBusy === issue.id}
              onClick={() => onRemovePhantom(issue)}
            >
              Supprimer
            </ActionButton>
          )}
        </div>
      </div>
    </div>
  );
}

export default DriverIssueCard;
