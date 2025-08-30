import { useState, useEffect } from "react"
import { invoke } from "@tauri-apps/api/core"
import ServiceCard from "./ServiceCard"
import "./App.css"

type ServiceStatus = {
  name: string
  running: boolean
}

type PortForwardConfig = {
  name: string
  context: string
  namespace: string
  service: string
  ports: string[]
}

function App() {
  let [configs, setConfigs] = useState<PortForwardConfig[]>([])
  let [services, setServices] = useState<ServiceStatus[]>([])
  let [message, setMessage] = useState("")
  let [loading, setLoading] = useState<string | null>(null)
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

  let loadConfigs = async () => {
    try {
      let loadedConfigs: PortForwardConfig[] = await invoke("get_port_forward_configs")
      setConfigs(loadedConfigs)
      setServices(loadedConfigs.map((config) => ({ name: config.name, running: false })))
    } catch (error) {
      console.error("Failed to load configs:", error)
      setMessage(`Error loading configs: ${error}`)
    }
  }

  let updateServiceStatus = async () => {
    try {
      let runningServices: string[] = await invoke("get_running_services")
      setServices((prev) =>
        prev.map((service) => ({
          ...service,
          running: runningServices.includes(service.name),
        })),
      )
    } catch (error) {
      console.error("Failed to get running services:", error)
    }
  }

  useEffect(() => {
    loadConfigs().then(updateServiceStatus)
  }, [])

  let setKubectlContext = async (context: string) => {
    setLoading("context")
    try {
      let result: string = await invoke("set_kubectl_context", { context })
      setMessage(result)
    } catch (error) {
      setMessage(`Error: ${error}`)
    } finally {
      setLoading(null)
    }
  }

  let startPortForward = async (serviceKey: string) => {
    setLoading(serviceKey)
    try {
      let result: string = await invoke("start_port_forward_by_key", { serviceKey })
      setMessage(result)
      await updateServiceStatus()
    } catch (error) {
      setMessage(`Error: ${error}`)
    } finally {
      setLoading(null)
    }
  }

  let addConfig = async (config: PortForwardConfig) => {
    try {
      await invoke("add_port_forward_config", { config })
      await loadConfigs()
      setMessage(`Added configuration for ${config.name}`)
    } catch (error) {
      setMessage(`Error adding config: ${error}`)
    }
  }

  let removeConfig = async (serviceKey: string) => {
    try {
      await invoke("remove_port_forward_config", { serviceKey })
      await loadConfigs()
      setMessage(`Removed configuration for ${serviceKey}`)
    } catch (error) {
      setMessage(`Error removing config: ${error}`)
    }
  }

  let updateConfig = async (oldServiceKey: string, newConfig: PortForwardConfig) => {
    try {
      await invoke("remove_port_forward_config", { serviceKey: oldServiceKey })
      await invoke("add_port_forward_config", { config: newConfig })
      await loadConfigs()
      setMessage(`Updated configuration for ${newConfig.name}`)
    } catch (error) {
      setMessage(`Error updating config: ${error}`)
    }
  }

  let loadContexts = async () => {
    try {
      let contexts: string[] = await invoke("get_kubectl_contexts")
      setAvailableContexts(contexts)
    } catch (error) {
      console.error("Failed to load contexts:", error)
    }
  }

  let loadNamespaces = async (context: string) => {
    if (!context) return
    try {
      let namespaces: string[] = await invoke("get_namespaces", { context })
      setAvailableNamespaces(namespaces)
    } catch (error) {
      console.error("Failed to load namespaces:", error)
      setAvailableNamespaces([])
    }
  }

  let loadServices = async (context: string, namespace: string) => {
    if (!context || !namespace) return
    try {
      let services: string[] = await invoke("get_services", { context, namespace })
      setAvailableServices(services)
    } catch (error) {
      console.error("Failed to load services:", error)
      setAvailableServices([])
    }
  }

  let loadPorts = async (context: string, namespace: string, service: string) => {
    if (!context || !namespace || !service) return
    try {
      let ports: string[] = await invoke("get_service_ports", { context, namespace, service })
      setAvailablePorts(ports)
    } catch (error) {
      console.error("Failed to load ports:", error)
      setAvailablePorts([])
    }
  }

  let stopPortForward = async (serviceName: string) => {
    setLoading(serviceName)
    try {
      let result: string = await invoke("stop_port_forward", { serviceName })
      setMessage(result)
      await updateServiceStatus()
    } catch (error) {
      let errorMessage = `${error}`
      setMessage(`Error: ${errorMessage}`)

      // If error indicates port forwarding is not running, reset the service state to stopped
      if (errorMessage.includes("port forwarding is not running")) {
        setServices((prev) =>
          prev.map((service) =>
            service.name === serviceName ? { ...service, running: false } : service,
          ),
        )
      }
    } finally {
      setLoading(null)
    }
  }

  return (
    <main className="container">
      <h1>Kubernetes Port Forwarding</h1>

      <div className="context-section">
        <h2>Kubectl Contexts</h2>
        <div className="context-buttons">
          <button
            onClick={() => setKubectlContext("hs-docn-cluster-1")}
            disabled={loading === "context"}
            className="context-button"
          >
            {loading === "context" ? "Setting Context..." : "hs-docn-cluster-1"}
          </button>
          <button
            onClick={() => setKubectlContext("tgs")}
            disabled={loading === "context"}
            className="context-button"
          >
            {loading === "context" ? "Setting Context..." : "tgs"}
          </button>
          <button
            onClick={() => setKubectlContext("hs-gcp-cluster-1")}
            disabled={loading === "context"}
            className="context-button"
          >
            {loading === "context" ? "Setting Context..." : "hs-gcp-cluster-1"}
          </button>
        </div>
      </div>

      <div className="services-section">
        <h2>Port Forwarding Services</h2>
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
                <input type="text" name="name" placeholder="Custom name for this configuration" required />
                <small>A friendly name to identify this port forward</small>
              </div>
              <div className="form-group">
                <label>Context:</label>
                <div className="input-with-detect">
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
                    {availableContexts.map(ctx => (
                      <option key={ctx} value={ctx}>{ctx}</option>
                    ))}
                  </select>
                  <button type="button" onClick={loadContexts} className="detect-button">
                    Detect
                  </button>
                </div>
                <input type="text" name="context" placeholder="Or enter manually" style={{marginTop: "0.5rem"}} />
              </div>
              <div className="form-group">
                <label>Namespace:</label>
                <div className="input-with-detect">
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
                    {availableNamespaces.map(ns => (
                      <option key={ns} value={ns}>{ns}</option>
                    ))}
                  </select>
                  <button 
                    type="button" 
                    onClick={() => loadNamespaces(selectedContext)} 
                    className="detect-button"
                    disabled={!selectedContext}
                  >
                    Detect
                  </button>
                </div>
                <input type="text" name="namespace" placeholder="Or enter manually" style={{marginTop: "0.5rem"}} />
              </div>
              <div className="form-group">
                <label>Service:</label>
                <div className="input-with-detect">
                  <select 
                    value={selectedService} 
                    onChange={(e) => {
                      setSelectedService(e.target.value)
                      loadPorts(selectedContext, selectedNamespace, e.target.value)
                    }}
                    disabled={!selectedContext || !selectedNamespace}
                  >
                    <option value="">Select service...</option>
                    {availableServices.map(svc => (
                      <option key={svc} value={svc}>{svc}</option>
                    ))}
                  </select>
                  <button 
                    type="button" 
                    onClick={() => loadServices(selectedContext, selectedNamespace)} 
                    className="detect-button"
                    disabled={!selectedContext || !selectedNamespace}
                  >
                    Detect
                  </button>
                </div>
                <input type="text" name="service" placeholder="Or enter manually (e.g., svc/my-service)" style={{marginTop: "0.5rem"}} />
              </div>
              <div className="form-group">
                <label>Ports:</label>
                {availablePorts.length > 0 && (
                  <div className="suggested-ports">
                    <small>Detected ports (click to use):</small>
                    <div className="port-suggestions">
                      {availablePorts.map(port => (
                        <button
                          key={port}
                          type="button"
                          className="port-suggestion"
                          onClick={() => {
                            let portsInput = document.querySelector('input[name="ports"]') as HTMLInputElement
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
                <button type="button" onClick={() => {
                  setShowAddForm(false)
                  setSelectedContext("")
                  setSelectedNamespace("")
                  setSelectedService("")
                  setAvailableContexts([])
                  setAvailableNamespaces([])
                  setAvailableServices([])
                  setAvailablePorts([])
                }}>
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
              let config = configs.find(c => c.name === activeServiceSettings)
              if (!config) return null
              let index = configs.findIndex(c => c.name === activeServiceSettings)
              return (
                <>
                  <h3>Settings for {config.name}</h3>
                  <div className="config-details">
                    <p><strong>Context:</strong> {config.context}</p>
                    <p><strong>Namespace:</strong> {config.namespace}</p>
                    <p><strong>Service:</strong> {config.service}</p>
                    <p><strong>Ports:</strong> {config.ports.join(", ")}</p>
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
                    <button 
                      onClick={() => setActiveServiceSettings(null)} 
                      className="close-button"
                    >
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
                <input
                  type="text"
                  name="name"
                  defaultValue={editingConfig.config.name}
                  required
                />
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
