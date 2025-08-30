import { useState } from "react"
import ServiceCard from "./ServiceCard"
import ServiceSettings from "./components/ServiceSettings"
import AddConfigForm from "./components/AddConfigForm"
import EditConfigForm from "./components/EditConfigForm"
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
        <AddConfigForm
          onAdd={(config) => {
            addConfig(config)
            setAvailableContexts([])
            setAvailableNamespaces([])
            setAvailableServices([])
            setAvailablePorts([])
          }}
          onClose={() => {
            setShowAddForm(false)
            setAvailableContexts([])
            setAvailableNamespaces([])
            setAvailableServices([])
            setAvailablePorts([])
          }}
          loadContexts={loadContexts}
          loadNamespaces={loadNamespaces}
          loadServices={loadServices}
          loadPorts={loadPorts}
          availableContexts={availableContexts}
          availableNamespaces={availableNamespaces}
          availableServices={availableServices}
          availablePorts={availablePorts}
        />
      )}

      <ServiceSettings
        config={configs.find((c) => c.name === activeServiceSettings) || null}
        onEdit={(config, index) => {
          setEditingConfig({ config, index })
          setShowConfigForm(true)
        }}
        onDelete={removeConfig}
        onClose={() => setActiveServiceSettings(null)}
        configs={configs}
      />

      <EditConfigForm
        editingConfig={editingConfig}
        onUpdate={(oldName, newConfig) => {
          updateConfig(oldName, newConfig)
          setEditingConfig(null)
        }}
        onClose={() => {
          setShowConfigForm(false)
          setEditingConfig(null)
        }}
      />

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
