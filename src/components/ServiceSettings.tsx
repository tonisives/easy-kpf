import { PortForwardConfig } from "../hooks/hooks"

type ServiceSettingsProps = {
  config: PortForwardConfig | null
  onEdit: (config: PortForwardConfig, index: number) => void
  onConfigureRecovery: (config: PortForwardConfig, index: number) => void
  onDelete: (configName: string) => void
  onClose: () => void
  configs: PortForwardConfig[]
}

let ServiceSettings = ({
  config,
  onEdit,
  onConfigureRecovery,
  onDelete,
  onClose,
  configs,
}: ServiceSettingsProps) => {
  if (!config) return null

  let index = configs.findIndex((c) => c.name === config.name)

  return (
    <div className="settings-modal">
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
            onClick={() => {
              onConfigureRecovery(config, index)
              onClose()
            }}
            className="recovery-button"
          >
            Recovery...
          </button>
          <button
            onClick={() => {
              onEdit(config, index)
              onClose()
            }}
            className="edit-button"
          >
            Edit...
          </button>
          <button
            onClick={() => {
              onDelete(config.name)
              onClose()
            }}
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
