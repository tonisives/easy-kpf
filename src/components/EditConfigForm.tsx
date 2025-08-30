import { PortForwardConfig } from "../hooks/hooks"

type EditConfigFormProps = {
  editingConfig: {
    config: PortForwardConfig
    index: number
  } | null
  onUpdate: (oldName: string, newConfig: PortForwardConfig) => void
  onClose: () => void
}

let EditConfigForm = ({ editingConfig, onUpdate, onClose }: EditConfigFormProps) => {
  if (!editingConfig) return null

  let handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    let formData = new FormData(e.target as HTMLFormElement)
    let portsString = formData.get("ports") as string
    let ports = portsString
      .split(",")
      .map((p) => p.trim())
      .filter((p) => p.length > 0)

    let updatedConfig: PortForwardConfig = {
      name: formData.get("name") as string,
      context: formData.get("context") as string,
      namespace: formData.get("namespace") as string,
      service: formData.get("service") as string,
      ports: ports,
    }

    onUpdate(editingConfig.config.name, updatedConfig)
    onClose()
  }

  return (
    <div className="add-form-modal">
      <div className="add-form">
        <h3>Edit Port Forward Configuration</h3>
        <form onSubmit={handleSubmit}>
          <div className="form-group">
            <label>Name:</label>
            <input type="text" name="name" defaultValue={editingConfig.config.name} required />
          </div>
          <div className="form-group">
            <label>Context:</label>
            <input
              type="text"
              name="context"
              defaultValue={editingConfig.config.context}
              required
            />
          </div>
          <div className="form-group">
            <label>Namespace:</label>
            <input
              type="text"
              name="namespace"
              defaultValue={editingConfig.config.namespace}
              required
            />
          </div>
          <div className="form-group">
            <label>Service:</label>
            <input
              type="text"
              name="service"
              defaultValue={editingConfig.config.service}
              placeholder="e.g., svc/my-service"
              required
            />
          </div>
          <div className="form-group">
            <label>Ports:</label>
            <input
              type="text"
              name="ports"
              defaultValue={editingConfig.config.ports.join(", ")}
              placeholder="e.g., 8080:80, 9090:3000"
              required
            />
            <small>Comma-separated list of local:remote ports</small>
          </div>
          <div className="form-actions">
            <button type="submit">Update Configuration</button>
            <button type="button" onClick={onClose}>
              Cancel
            </button>
          </div>
        </form>
      </div>
    </div>
  )
}

export default EditConfigForm