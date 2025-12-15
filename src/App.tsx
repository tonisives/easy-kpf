import { useState, useEffect, useRef, useMemo } from "react"
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
  sortableKeyboardCoordinates,
} from "@dnd-kit/sortable"
import ServiceCard from "./ServiceCard"
import ServiceSettings from "./components/ServiceSettings"
import AddConfigForm from "./components/AddConfigForm"
import SetupScreen from "./components/SetupScreen"
import ContextAccordion from "./components/ContextAccordion"
import "./App.css"
import { PortForwardConfig, useConfigs } from "./hooks/hooks"
import { groupConfigsByContext } from "./utils/groupingUtils"

function App() {
  let [message, setMessage] = useState("")
  let [kubectlConfigured, setKubectlConfigured] = useState<boolean | null>(null)
  let [showSettings, setShowSettings] = useState(false)

  let [showAddForm, setShowAddForm] = useState(false)
  let [activeServiceSettings, setActiveServiceSettings] = useState<string | null>(null)
  let [_, setShowConfigForm] = useState(false)
  let [editingConfig, setEditingConfig] = useState<{
    config: PortForwardConfig
    index: number
  } | null>(null)
  let [activeId, setActiveId] = useState<string | null>(null)
  let [searchQuery, setSearchQuery] = useState("")
  let [searchFocused, setSearchFocused] = useState(false)
  let searchInputRef = useRef<HTMLInputElement>(null)
  let {
    configs,
    services,
    loading,
    formError,
    startPortForward,
    addConfig,
    removeConfig,
    updateConfig,
    reorderConfig,
    stopPortForward,
    clearServiceError,
    clearFormError,
    reconnectAll,
  } = useConfigs(setMessage, () => {}, () => {}, () => {}, () => {})

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

  // Keyboard shortcut for search (/ or Cmd+F)
  useEffect(() => {
    let handleKeyDown = (e: KeyboardEvent) => {
      // Don't trigger if we're in an input, textarea, or contenteditable
      let target = e.target as HTMLElement
      if (
        target.tagName === "INPUT" ||
        target.tagName === "TEXTAREA" ||
        target.isContentEditable
      ) {
        // Allow Escape to blur the search input
        if (e.key === "Escape" && target === searchInputRef.current) {
          searchInputRef.current?.blur()
          setSearchFocused(false)
        }
        return
      }

      // / or Cmd+F to focus search
      if (e.key === "/" || (e.key === "f" && (e.metaKey || e.ctrlKey))) {
        e.preventDefault()
        searchInputRef.current?.focus()
        setSearchFocused(true)
      }

      // Escape to clear search when not focused
      if (e.key === "Escape" && searchQuery) {
        setSearchQuery("")
      }
    }

    window.addEventListener("keydown", handleKeyDown)
    return () => window.removeEventListener("keydown", handleKeyDown)
  }, [searchQuery])

  // Filter configs based on search query
  let filteredConfigs = useMemo(() => {
    if (!searchQuery.trim()) return configs

    let query = searchQuery.toLowerCase()
    return configs.filter(
      (c) =>
        c.name.toLowerCase().includes(query) ||
        c.service.toLowerCase().includes(query) ||
        c.namespace.toLowerCase().includes(query) ||
        c.context.toLowerCase().includes(query) ||
        c.ports.some((p) => p.includes(query))
    )
  }, [configs, searchQuery])

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

  if (!kubectlConfigured || showSettings) {
    return <SetupScreen 
      onSetupComplete={() => {
        setKubectlConfigured(true)
        setShowSettings(false)
      }}
      onCancel={showSettings ? () => setShowSettings(false) : undefined}
      isDialog={showSettings}
    />
  }

  return (
    <main className="container">
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
        <h1 style={{}}>Kubernetes Port Forwarding</h1>
        <div style={{ display: "flex", gap: "10px" }}>
          <button
            onClick={() => reconnectAll()}
            className="add-button"
            disabled={!services.some((s) => !s.running && s.error)}
            title="Reconnect all disconnected services"
          >
            Reconnect All
          </button>
          <button onClick={() => setShowAddForm(true)} className="add-button">
            Add New Configuration
          </button>
          <button onClick={() => setShowSettings(true)} className="settings-icon-button" title="Settings">
            ⚙️
          </button>
        </div>
      </div>

      <div className="search-container">
        <div className={`search-input-wrapper ${searchFocused ? "focused" : ""}`}>
          <span className="search-icon">⌕</span>
          <input
            ref={searchInputRef}
            type="text"
            className="search-input"
            placeholder="Filter port forwards... (/ or Cmd+F)"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            onFocus={() => setSearchFocused(true)}
            onBlur={() => setSearchFocused(false)}
          />
          {searchQuery && (
            <button
              className="search-clear"
              onClick={() => {
                setSearchQuery("")
                searchInputRef.current?.focus()
              }}
              title="Clear search (Esc)"
            >
              x
            </button>
          )}
        </div>
        {searchQuery && (
          <span className="search-results-count">
            {filteredConfigs.length} of {configs.length} shown
          </span>
        )}
      </div>

      <div className="services-section">
        <DndContext
          sensors={sensors}
          collisionDetection={closestCenter}
          onDragStart={handleDragStart}
          onDragEnd={handleDragEnd}
        >
          {groupConfigsByContext(filteredConfigs).map((group) => (
            <ContextAccordion
              key={group.context}
              group={group}
              services={services}
              loading={loading}
              onStart={startPortForward}
              onStop={stopPortForward}
              onSettings={setActiveServiceSettings}
              onClearError={clearServiceError}
            />
          ))}
          <DragOverlay>
            {activeId
              ? (() => {
                  let config = configs.find((c) => c.name === activeId)
                  let service = services.find((s) => s.name === activeId)

                  if (!config) return null

                  // Format display differently for SSH vs Kubectl
                  let displayInfo = config.forward_type === "Ssh"
                    ? {
                        displayName: config.name,
                        context: config.service, // SSH host
                        namespace: config.forward_type, // Show type instead of namespace
                        ports: `Ports: ${config.ports.join(", ")}`
                      }
                    : {
                        displayName: `${config.name} (${config.service})`,
                        context: config.context,
                        namespace: config.namespace,
                        ports: `Ports: ${config.ports.join(", ")}`
                      }

                  return (
                    <ServiceCard
                      id={config.name}
                      name={config.name}
                      displayName={displayInfo.displayName}
                      context={displayInfo.context}
                      namespace={displayInfo.namespace}
                      ports={displayInfo.ports}
                      isRunning={service?.running || false}
                      isLoading={loading === config.name}
                      error={service?.error}
                      onStart={() => startPortForward(config.name)}
                      onStop={() => stopPortForward(config.name)}
                      onSettings={() => setActiveServiceSettings(config.name)}
                      onClearError={() => clearServiceError(config.name)}
                    />
                  )
                })()
              : null}
          </DragOverlay>
        </DndContext>
      </div>

      {showAddForm && (
        <AddConfigForm
          onAdd={addConfig}
          onClose={() => {
            setShowAddForm(false)
            clearFormError()
          }}
          error={formError}
          onClearError={clearFormError}
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

      {editingConfig && (
        <AddConfigForm
          editingConfig={editingConfig}
          onAdd={() => {}}
          onUpdate={(oldName, newConfig) => {
            updateConfig(oldName, newConfig)
            setEditingConfig(null)
          }}
          onClose={() => {
            setShowConfigForm(false)
            setEditingConfig(null)
            clearFormError()
          }}
          error={formError}
          onClearError={clearFormError}
        />
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
