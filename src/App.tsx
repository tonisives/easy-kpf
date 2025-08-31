import { useState, useEffect } from "react"
import { invoke } from "@tauri-apps/api/core"
import ServiceCard from "./ServiceCard"
import ServiceSettings from "./components/ServiceSettings"
import AddConfigForm from "./components/AddConfigForm"
import EditConfigForm from "./components/EditConfigForm"
import SetupScreen from "./components/SetupScreen"
import "./App.css"
import { PortForwardConfig, useConfigs } from "./hooks/hooks"

function App() {
  let [message, setMessage] = useState("")
  let [kubectlConfigured, setKubectlConfigured] = useState<boolean | null>(null)

  let [showAddForm, setShowAddForm] = useState(false)
  let [activeServiceSettings, setActiveServiceSettings] = useState<string | null>(null)
  let [_, setShowConfigForm] = useState(false)
  let [editingConfig, setEditingConfig] = useState<{
    config: PortForwardConfig
    index: number
  } | null>(null)
  let [availableContexts, setAvailableContexts] = useState<string[]>([])
  let [availableNamespaces, setAvailableNamespaces] = useState<string[]>([])
  let [availableServices, setAvailableServices] = useState<string[]>([])
  let [availablePorts, setAvailablePorts] = useState<string[]>([])
  let [draggedIndex, setDraggedIndex] = useState<number | null>(null)
  let [dragOverIndex, setDragOverIndex] = useState<number | null>(null)
  let {
    configs,
    services,
    loading,
    startPortForward,
    addConfig,
    removeConfig,
    updateConfig,
    reorderConfig,
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

  useEffect(() => {
    let checkKubectlSetup = async () => {
      try {
        let path = await invoke<string | null>("get_kubectl_path")
        setKubectlConfigured(!!path)
      } catch {
        setKubectlConfigured(false)
      }
    }
    checkKubectlSetup()
  }, [])

  let handleDragStart = (index: number) => (e: React.DragEvent<HTMLDivElement>) => {
    console.log('Drag started:', index)
    setDraggedIndex(index)
    e.dataTransfer.effectAllowed = "move"
    e.dataTransfer.setData('text/plain', index.toString())
  }

  let handleDragOver = (index: number) => (e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault()
    e.dataTransfer.dropEffect = "move"
    
    if (draggedIndex === null || draggedIndex === index) {
      return
    }
    
    setDragOverIndex(index)
  }

  let handleDragLeave = () => {
    setDragOverIndex(null)
  }

  let handleDrop = (index: number) => async (e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault()
    setDragOverIndex(null)
    
    console.log('Drop event:', { draggedIndex, targetIndex: index })
    
    if (draggedIndex !== null && draggedIndex !== index) {
      let config = configs[draggedIndex]
      console.log('Reordering:', config.name, 'from', draggedIndex, 'to', index)
      
      try {
        await reorderConfig(config.name, index)
        console.log('Reorder completed successfully')
      } catch (error) {
        console.error('Reorder failed:', error)
      }
    }
    
    setDraggedIndex(null)
  }

  if (kubectlConfigured === null) {
    return <div style={{ padding: "20px", textAlign: "center" }}>Loading...</div>
  }

  if (!kubectlConfigured) {
    return <SetupScreen onSetupComplete={() => setKubectlConfigured(true)} />
  }

  return (
    <main className="container">
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
        <h1 style={{}}>Kubernetes Port Forwarding</h1>
        <div>
          <button onClick={() => setShowAddForm(true)} className="add-button">
            Add New Configuration
          </button>
        </div>
      </div>

      <div style={{ height: "20px" }} />

      <div className="services-section" onDragLeave={handleDragLeave}>
        {configs.map((config, index) => {
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
              draggable={true}
              onDragStart={handleDragStart(index)}
              onDragOver={handleDragOver(index)}
              onDrop={handleDrop(index)}
              isDragOver={dragOverIndex === index}
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
