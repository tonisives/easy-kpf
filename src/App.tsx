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
  DragOverlay,
  DragStartEvent,
} from "@dnd-kit/core"
import {
  SortableContext,
  sortableKeyboardCoordinates,
  verticalListSortingStrategy,
} from "@dnd-kit/sortable"
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
  let [activeId, setActiveId] = useState<string | null>(null)
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
    reorderConfig,
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
    }),
  )

  let handleDragStart = (event: DragStartEvent) => {
    setActiveId(event.active.id as string)
  }

  let handleDragEnd = (event: DragEndEvent) => {
    let { active, over } = event

    if (active.id !== over?.id) {
      let oldIndex = configs.findIndex((config) => config.name === active.id)
      let newIndex = configs.findIndex((config) => config.name === over?.id)

      if (oldIndex !== -1 && newIndex !== -1) {
        let config = configs[oldIndex]
        console.log("Reordering:", config.name, "from", oldIndex, "to", newIndex)

        reorderConfig(config.name, newIndex).catch((error: any) => {
          console.error("Reorder failed:", error)
        })
      }
    }

    setActiveId(null)
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
          onDragStart={handleDragStart}
          onDragEnd={handleDragEnd}
        >
          <SortableContext
            items={configs.map((config) => config.name)}
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
          <DragOverlay>
            {activeId
              ? (() => {
                  let config = configs.find((c) => c.name === activeId)
                  let service = services.find((s) => s.name === activeId)
                  return config ? (
                    <ServiceCard
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
                  ) : null
                })()
              : null}
          </DragOverlay>
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
