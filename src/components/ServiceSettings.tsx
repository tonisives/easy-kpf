import { useEffect, useState, type MouseEvent } from "react"
import { PortForwardConfig, RecoveryScopes } from "../hooks/hooks"

type ServiceSettingsProps = {
  config: PortForwardConfig | null
  errors?: string[]
  onEdit: (config: PortForwardConfig, index: number) => void
  onConfigureRecovery: (config: PortForwardConfig, index: number) => void
  onDelete: (configName: string) => void
  onClearErrors: () => void
  onClose: () => void
  configs: PortForwardConfig[]
  recoveryScopes: RecoveryScopes
}

let ServiceSettings = ({
  config,
  errors,
  onEdit,
  onConfigureRecovery,
  onDelete,
  onClearErrors,
  onClose,
  configs,
  recoveryScopes,
}: ServiceSettingsProps) => {
  let [showLogs, setShowLogs] = useState(false)

  useEffect(() => {
    setShowLogs(false)
  }, [config?.name])

  if (!config) return null

  let index = configs.findIndex((c) => c.name === config.name)
  let hasLogs = Boolean(errors?.length)

  let handleBackdropClick = (event: MouseEvent<HTMLDivElement>) => {
    if (event.target === event.currentTarget) {
      onClose()
    }
  }

  let handleConfigureRecovery = () => {
    onConfigureRecovery(config, index)
    onClose()
  }

  let handleEdit = () => {
    onEdit(config, index)
    onClose()
  }

  let handleDelete = () => {
    onDelete(config.name)
    onClose()
  }

  let handleShowLogs = () => {
    setShowLogs(true)
  }

  let handleHideLogs = () => {
    setShowLogs(false)
  }

  if (showLogs) {
    return (
      <div className="settings-modal" onClick={handleBackdropClick}>
        <div className="service-settings-popup">
          <div className="dialog-heading">
            <h3>{config.name} Logs</h3>
            <p>Recent connection and recovery errors</p>
          </div>
          {hasLogs ? (
            <div className="service-logs">
              {errors?.map((error, index) => (
                <div key={index} className="service-log-line">{error}</div>
              ))}
            </div>
          ) : (
            <p className="service-logs-empty">No recent errors.</p>
          )}
          <div className="service-settings-actions">
            {hasLogs && (
              <button onClick={onClearErrors} className="delete-button">
                Clear Logs
              </button>
            )}
            <button onClick={handleHideLogs} className="cancel-button">
              Back
            </button>
          </div>
        </div>
      </div>
    )
  }

  return (
    <div className="settings-modal" onClick={handleBackdropClick}>
      <div className="service-settings-popup">
        <div className="dialog-heading">
          <h3>{config.name}</h3>
          <p>Port forward configuration</p>
        </div>
        <div className="config-details">
          <p>
            <strong>Context</strong><span>{config.context}</span>
          </p>
          <p>
            <strong>Namespace</strong><span>{config.namespace}</span>
          </p>
          <p>
            <strong>Service</strong><span>{config.service}</span>
          </p>
          <p>
            <strong>Ports</strong><span>{config.ports.join(", ")}</span>
          </p>
          <p>
            <strong>Recovery</strong>
            <span>
              {config.recovery
                ? config.recovery.reconnect.enabled
                  ? "Custom, enabled"
                  : "Custom, disabled"
                : config.forward_type === "Ssh" && recoveryScopes.ssh_hosts[config.context]
                  ? "SSH host settings"
                  : config.forward_type === "Kubectl"
                    && recoveryScopes.kubernetes_contexts[config.context]
                    ? "Kubernetes context settings"
                    : "Built-in defaults"}
            </span>
          </p>
          {config.recovery?.hooks.before_reconnect[0] && (
            <p>
              <strong>Recovery Hook</strong>
              <span>
                {config.recovery.hooks.before_reconnect[0].type === "ssh"
                  ? `SSH ${config.recovery.hooks.before_reconnect[0].ssh_host || ""}`
                  : config.recovery.hooks.before_reconnect[0].command}
              </span>
            </p>
          )}
        </div>
        <div className="service-settings-actions">
          <button
            onClick={handleShowLogs}
            className="logs-button"
          >
            Logs{hasLogs ? ` (${errors?.length})` : ""}
          </button>
          <button
            onClick={handleConfigureRecovery}
            className="recovery-button"
          >
            Recovery...
          </button>
          <button
            onClick={handleEdit}
            className="edit-button"
          >
            Edit...
          </button>
          <button
            onClick={handleDelete}
            className="delete-button"
          >
            Delete
          </button>
          <button onClick={onClose} className="cancel-button">
            Cancel
          </button>
        </div>
      </div>
    </div>
  )
}

export default ServiceSettings
