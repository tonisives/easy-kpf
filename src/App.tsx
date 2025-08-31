import { useState, useEffect } from "react"
import { invoke } from "@tauri-apps/api/core"
import {
  DndContext,
  closestCenter,
  KeyboardSensor,
  PointerSensor,
  useSensor,
  useSensors,
  DragEndEvent,
} from '@dnd-kit/core'
import {
  SortableContext,
  sortableKeyboardCoordinates,
  verticalListSortingStrategy,
} from '@dnd-kit/sortable'
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

  let sensors = useSensors(
    useSensor(PointerSensor),
    useSensor(KeyboardSensor, {
      coordinateGetter: sortableKeyboardCoordinates,
    })
  )

  let handleDragEnd = async (event: DragEndEvent) => {
    let { active, over } = event

    if (active.id !== over?.id) {
      let oldIndex = configs.findIndex(config => config.name === active.id)
      let newIndex = configs.findIndex(config => config.name === over?.id)
      
      if (oldIndex !== -1 && newIndex !== -1) {
        let config = configs[oldIndex]
        console.log('Reordering:', config.name, 'from', oldIndex, 'to', newIndex)
        
        try {
          await reorderConfig(config.name, newIndex)
          console.log('Reorder completed successfully')
        } catch (error) {
          console.error('Reorder failed:', error)
        }
      }
    }
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

      <div className="services-section">
        <DndContext
          sensors={sensors}
          collisionDetection={closestCenter}
          onDragEnd={handleDragEnd}
        >
          <SortableContext
            items={configs.map(config => config.name)}
            strategy={verticalListSortingStrategy}
          >
            {configs.map((config) => {
              let service = services.find((s) => s.name === config.name)
              return (
                <ServiceCard
                  key={config.name}
                  id={config.name}
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
          </SortableContext>
        </DndContext>
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
