import { useState } from "react"
import ServiceCard from "./ServiceCard"
import "./App.css"
import { PortForwardConfig, useConfigs } from "./hooks/hooks"

function App() {
  let [message, setMessage] = useState("")

  let [showAddForm, setShowAddForm] = useState(false)
  let [activeServiceSettings, setActiveServiceSettings] = useState<string | null>(null)
  let [showConfigForm, setShowConfigForm] = useState(false)
  let [editingConfig, setEditingConfig] = useState<{
    config: PortForwardConfig
    index: number
  } | null>(null)
  let [availableContexts, setAvailableContexts] = useState<string[]>([])
  let [availableNamespaces, setAvailableNamespaces] = useState<string[]>([])
  let [availableServices, setAvailableServices] = useState<string[]>([])
  let [availablePorts, setAvailablePorts] = useState<string[]>([])
  let [selectedContext, setSelectedContext] = useState("")
  let [selectedNamespace, setSelectedNamespace] = useState("")
  let [selectedService, setSelectedService] = useState("")
  let {
    configs,
    services,
    loading,
    startPortForward,
    addConfig,
    removeConfig,
    updateConfig,
    loadContexts,
    loadNamespaces,
    loadServices,
    loadPorts,
    stopPortForward,
  } = useConfigs(
    setMessage,
    setAvailablePorts,
    setAvailableContexts,
    setAvailableNamespaces,
    setAvailableServices,
  )

  return (
    <main className="container">
      <h1>Kubernetes Port Forwarding</h1>

      <div className="services-section">
        <div className="controls-section">
          <button onClick={() => setShowAddForm(true)} className="add-button">
            Add New Configuration
          </button>
        </div>

        {configs.map((config) => {
          let service = services.find((s) => s.name === config.name)
          return (
            <ServiceCard
              key={config.name}
              name={config.name}
              displayName={`${config.name} (${config.service})`}
              context={config.context}
              namespace={config.namespace}
              ports={`Ports: ${config.ports.join(", ")}`}
              isRunning={service?.running || false}
              isLoading={loading === config.name}
              onStart={() => startPortForward(config.name)}
              onStop={() => stopPortForward(config.name)}
              onSettings={() => setActiveServiceSettings(config.name)}
            />
          )
        })}
      </div>

      {showAddForm && (
        <div className="add-form-modal">
          <div className="add-form">
            <h3>Add New Port Forward Configuration</h3>
            <form
              onSubmit={(e) => {
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

                addConfig(newConfig)
                setShowAddForm(false)
                setSelectedContext("")
                setSelectedNamespace("")
                setSelectedService("")
                setAvailableContexts([])
                setAvailableNamespaces([])
                setAvailableServices([])
                setAvailablePorts([])
              }}
            >
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
                        setAvailableServices([])
                        setAvailablePorts([])
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
                        setAvailablePorts([])
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
                <button
                  type="button"
                  onClick={() => {
                    setShowAddForm(false)
                    setSelectedContext("")
                    setSelectedNamespace("")
                    setSelectedService("")
                    setAvailableContexts([])
                    setAvailableNamespaces([])
                    setAvailableServices([])
                    setAvailablePorts([])
                  }}
                >
                  Cancel
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {activeServiceSettings && (
        <div className="settings-modal">
          <div className="service-settings-popup">
            {(() => {
              let config = configs.find((c) => c.name === activeServiceSettings)
              if (!config) return null
              let index = configs.findIndex((c) => c.name === activeServiceSettings)
              return (
                <>
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
                        setEditingConfig({ config, index })
                        setShowConfigForm(true)
                        setActiveServiceSettings(null)
                      }}
                      className="edit-button"
                    >
                      Edit Configuration
                    </button>
                    <button
                      onClick={() => {
                        removeConfig(config.name)
                        setActiveServiceSettings(null)
                      }}
                      className="delete-button"
                    >
                      Delete Configuration
                    </button>
                    <button onClick={() => setActiveServiceSettings(null)} className="close-button">
                      Close
                    </button>
                  </div>
                </>
              )
            })()}
          </div>
        </div>
      )}

      {showConfigForm && editingConfig && (
        <div className="add-form-modal">
          <div className="add-form">
            <h3>Edit Port Forward Configuration</h3>
            <form
              onSubmit={(e) => {
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

                updateConfig(editingConfig.config.name, updatedConfig)
                setShowConfigForm(false)
                setEditingConfig(null)
              }}
            >
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
                <button
                  type="button"
                  onClick={() => {
                    setShowConfigForm(false)
                    setEditingConfig(null)
                  }}
                >
                  Cancel
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {message && (
        <div className="message">
          <pre>{message}</pre>
          <button onClick={() => setMessage("")} className="clear-button">
            Clear
          </button>
        </div>
      )}
    </main>
  )
}

export default App
