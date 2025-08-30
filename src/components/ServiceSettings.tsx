import { PortForwardConfig } from "../hooks/hooks"

type ServiceSettingsProps = {
  config: PortForwardConfig | null
  onEdit: (config: PortForwardConfig, index: number) => void
  onDelete: (configName: string) => void
  onClose: () => void
  configs: PortForwardConfig[]
}

let ServiceSettings = ({ config, onEdit, onDelete, onClose, configs }: ServiceSettingsProps) => {
  if (!config) return null

  let index = configs.findIndex((c) => c.name === config.name)

  return (
    <div className="settings-modal">
      <div className="service-settings-popup">
        <h3>Settings for {config.name}</h3>
        <div className="config-details">
          <p>
            <strong>Context:</strong> {config.context}
          </p>
          <p>
            <strong>Namespace:</strong> {config.namespace}
          </p>
          <p>
            <strong>Service:</strong> {config.service}
          </p>
          <p>
            <strong>Ports:</strong> {config.ports.join(", ")}
          </p>
        </div>
        <div className="service-settings-actions">
          <button
            onClick={() => {
              onEdit(config, index)
              onClose()
            }}
            className="edit-button"
          >
            Edit Configuration
          </button>
          <button
            onClick={() => {
              onDelete(config.name)
              onClose()
            }}
            className="delete-button"
          >
            Delete Configuration
          </button>
          <button onClick={onClose} className="close-button">
            Close
          </button>
        </div>
      </div>
    </div>
  )
}

export default ServiceSettings