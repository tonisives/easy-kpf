import { useState } from "react"
import { PortForwardConfig } from "../hooks/hooks"

type AddConfigFormProps = {
  onAdd: (config: PortForwardConfig) => void
  onClose: () => void
  loadContexts: () => void
  loadNamespaces: (context: string) => void
  loadServices: (context: string, namespace: string) => void
  loadPorts: (context: string, namespace: string, service: string) => void
  availableContexts: string[]
  availableNamespaces: string[]
  availableServices: string[]
  availablePorts: string[]
}

let AddConfigForm = ({
  onAdd,
  onClose,
  loadContexts,
  loadNamespaces,
  loadServices,
  loadPorts,
  availableContexts,
  availableNamespaces,
  availableServices,
  availablePorts,
}: AddConfigFormProps) => {
  let [selectedContext, setSelectedContext] = useState("")
  let [selectedNamespace, setSelectedNamespace] = useState("")
  let [selectedService, setSelectedService] = useState("")

  let handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    let formData = new FormData(e.target as HTMLFormElement)
    let portsString = formData.get("ports") as string
    let ports = portsString
      .split(",")
      .map((p) => p.trim())
      .filter((p) => p.length > 0)

    let newConfig: PortForwardConfig = {
      name: formData.get("name") as string,
      context: selectedContext || (formData.get("context") as string),
      namespace: selectedNamespace || (formData.get("namespace") as string),
      service: selectedService || (formData.get("service") as string),
      ports: ports,
    }

    onAdd(newConfig)
    onClose()
  }

  let handleCancel = () => {
    setSelectedContext("")
    setSelectedNamespace("")
    setSelectedService("")
    onClose()
  }

  return (
    <div className="add-form-modal">
      <div className="add-form">
        <h3>Add New Port Forward Configuration</h3>
        <form onSubmit={handleSubmit}>
          <div className="form-group">
            <label>Name:</label>
            <input
              type="text"
              name="name"
              placeholder="Custom name for this configuration"
              required
            />
            <small>A friendly name to identify this port forward</small>
          </div>
          <div className="form-group">
            <label>Context:</label>
            <div className="input-with-detect">
              <div className="input-section">
                <select
                  value={selectedContext}
                  onChange={(e) => {
                    setSelectedContext(e.target.value)
                    loadNamespaces(e.target.value)
                    setSelectedNamespace("")
                    setSelectedService("")
                  }}
                >
                  <option value="">Select context...</option>
                  {availableContexts.map((ctx) => (
                    <option key={ctx} value={ctx}>
                      {ctx}
                    </option>
                  ))}
                </select>
                <input type="text" name="context" placeholder="Or enter manually" />
              </div>
              <button type="button" onClick={loadContexts} className="detect-button">
                Detect
              </button>
            </div>
          </div>
          <div className="form-group">
            <label>Namespace:</label>
            <div className="input-with-detect">
              <div className="input-section">
                <select
                  value={selectedNamespace}
                  onChange={(e) => {
                    setSelectedNamespace(e.target.value)
                    loadServices(selectedContext, e.target.value)
                    setSelectedService("")
                  }}
                  disabled={!selectedContext}
                >
                  <option value="">Select namespace...</option>
                  {availableNamespaces.map((ns) => (
                    <option key={ns} value={ns}>
                      {ns}
                    </option>
                  ))}
                </select>
                <input type="text" name="namespace" placeholder="Or enter manually" />
              </div>
              <button
                type="button"
                onClick={() => loadNamespaces(selectedContext)}
                className="detect-button"
                disabled={!selectedContext}
              >
                Detect
              </button>
            </div>
          </div>
          <div className="form-group">
            <label>Service:</label>
            <div className="input-with-detect">
              <div className="input-section">
                <select
                  value={selectedService}
                  onChange={(e) => {
                    setSelectedService(e.target.value)
                    loadPorts(selectedContext, selectedNamespace, e.target.value)
                  }}
                  disabled={!selectedContext || !selectedNamespace}
                >
                  <option value="">Select service...</option>
                  {availableServices.map((svc) => (
                    <option key={svc} value={svc}>
                      {svc}
                    </option>
                  ))}
                </select>
                <input
                  type="text"
                  name="service"
                  placeholder="Or enter manually (e.g., svc/my-service)"
                />
              </div>
              <button
                type="button"
                onClick={() => loadServices(selectedContext, selectedNamespace)}
                className="detect-button"
                disabled={!selectedContext || !selectedNamespace}
              >
                Detect
              </button>
            </div>
          </div>
          <div className="form-group">
            <label>Ports:</label>
            {availablePorts.length > 0 && (
              <div className="suggested-ports">
                <small>Detected ports (click to use):</small>
                <div className="port-suggestions">
                  {availablePorts.map((port) => (
                    <button
                      key={port}
                      type="button"
                      className="port-suggestion"
                      onClick={() => {
                        let portsInput = document.querySelector(
                          'input[name="ports"]',
                        ) as HTMLInputElement
                        if (portsInput) {
                          let current = portsInput.value.trim()
                          portsInput.value = current ? `${current}, ${port}` : port
                        }
                      }}
                    >
                      {port}
                    </button>
                  ))}
                </div>
              </div>
            )}
            <input type="text" name="ports" placeholder="e.g., 8080:80, 9090:3000" required />
            <small>Comma-separated list of local:remote ports</small>
          </div>
          <div className="form-actions">
            <button type="submit">Add Configuration</button>
            <button type="button" onClick={handleCancel}>
              Cancel
            </button>
          </div>
        </form>
      </div>
    </div>
  )
}

export default AddConfigForm