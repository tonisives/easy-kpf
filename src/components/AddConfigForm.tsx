import { useState, useEffect } from "react"
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

  // Auto-load contexts on mount
  useEffect(() => {
    loadContexts()
  }, [])

  // Auto-load namespaces when context changes
  useEffect(() => {
    if (selectedContext) {
      loadNamespaces(selectedContext)
      setSelectedNamespace("")
      setSelectedService("")
    }
  }, [selectedContext])

  // Auto-load services when namespace changes
  useEffect(() => {
    if (selectedContext && selectedNamespace) {
      loadServices(selectedContext, selectedNamespace)
      setSelectedService("")
    }
  }, [selectedNamespace])

  // Auto-load ports when service changes
  useEffect(() => {
    if (selectedContext && selectedNamespace && selectedService) {
      loadPorts(selectedContext, selectedNamespace, selectedService)
    }
  }, [selectedService])

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
      context: selectedContext,
      namespace: selectedNamespace,
      service: selectedService,
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
            <select
              value={selectedContext}
              onChange={(e) => setSelectedContext(e.target.value)}
              required
            >
              <option value="">Select context...</option>
              {availableContexts.map((ctx) => (
                <option key={ctx} value={ctx}>
                  {ctx}
                </option>
              ))}
            </select>
          </div>
          <div className="form-group">
            <label>Namespace:</label>
            <select
              value={selectedNamespace}
              onChange={(e) => setSelectedNamespace(e.target.value)}
              disabled={!selectedContext}
              required
            >
              <option value="">Select namespace...</option>
              {availableNamespaces.map((ns) => (
                <option key={ns} value={ns}>
                  {ns}
                </option>
              ))}
            </select>
          </div>
          <div className="form-group">
            <label>Service:</label>
            <select
              value={selectedService}
              onChange={(e) => setSelectedService(e.target.value)}
              disabled={!selectedContext || !selectedNamespace}
              required
            >
              <option value="">Select service...</option>
              {availableServices.map((svc) => (
                <option key={svc} value={svc}>
                  {svc}
                </option>
              ))}
            </select>
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