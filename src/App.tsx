import { useState, useEffect } from "react"
import { invoke } from "@tauri-apps/api/core"
import ServiceCard from "./ServiceCard"
import "./App.css"

type ServiceStatus = {
  name: string
  running: boolean
}

type PortForwardConfig = {
  service_key: string
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

  let loadConfigs = async () => {
    try {
      let loadedConfigs: PortForwardConfig[] = await invoke("get_port_forward_configs")
      setConfigs(loadedConfigs)
      setServices(loadedConfigs.map(config => ({ name: config.service_key, running: false })))
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
      setMessage(`Added configuration for ${config.service_key}`)
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
        <div className="add-config-section">
          <button 
            onClick={() => setShowAddForm(true)}
            className="add-button"
          >
            Add New Configuration
          </button>
        </div>

        {configs.map((config) => {
          let service = services.find((s) => s.name === config.service_key)
          return (
            <div key={config.service_key} className="service-with-controls">
              <ServiceCard
                name={config.service_key}
                displayName={`${config.service_key} (${config.service})`}
                context={config.context}
                namespace={config.namespace}
                ports={`Ports: ${config.ports.join(", ")}`}
                isRunning={service?.running || false}
                isLoading={loading === config.service_key}
                onStart={() => startPortForward(config.service_key)}
                onStop={() => stopPortForward(config.service_key)}
              />
              <button 
                onClick={() => removeConfig(config.service_key)}
                className="remove-button"
                title="Remove configuration"
              >
                Remove
              </button>
            </div>
          )
        })}
      </div>

      {showAddForm && (
        <div className="add-form-modal">
          <div className="add-form">
            <h3>Add New Port Forward Configuration</h3>
            <form onSubmit={(e) => {
              e.preventDefault()
              let formData = new FormData(e.target as HTMLFormElement)
              let portsString = formData.get("ports") as string
              let ports = portsString.split(",").map(p => p.trim()).filter(p => p.length > 0)
              
              let newConfig: PortForwardConfig = {
                service_key: formData.get("service_key") as string,
                context: formData.get("context") as string,
                namespace: formData.get("namespace") as string,
                service: formData.get("service") as string,
                ports: ports
              }
              
              addConfig(newConfig)
              setShowAddForm(false)
            }}>
              <div className="form-group">
                <label>Service Key:</label>
                <input type="text" name="service_key" required />
              </div>
              <div className="form-group">
                <label>Context:</label>
                <input type="text" name="context" required />
              </div>
              <div className="form-group">
                <label>Namespace:</label>
                <input type="text" name="namespace" required />
              </div>
              <div className="form-group">
                <label>Service:</label>
                <input type="text" name="service" placeholder="e.g., svc/my-service" required />
              </div>
              <div className="form-group">
                <label>Ports:</label>
                <input type="text" name="ports" placeholder="e.g., 8080:80, 9090:3000" required />
                <small>Comma-separated list of local:remote ports</small>
              </div>
              <div className="form-actions">
                <button type="submit">Add Configuration</button>
                <button type="button" onClick={() => setShowAddForm(false)}>Cancel</button>
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
