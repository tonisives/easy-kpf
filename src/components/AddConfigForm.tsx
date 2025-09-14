import { PortForwardConfig } from "../hooks/hooks"
import { useKubernetesDataFlow } from "../hooks/useKubernetesDataFlow"
import { useFormState, deriveConfigName } from "../hooks/useFormState"
import { KubernetesSelect } from "./KubernetesSelect"
import { PortSuggestions } from "./PortSuggestions"
import { ErrorBanner } from "./ErrorBanner"
import { FormActions } from "./FormActions"
import { useState, useEffect, useRef } from "react"
import { invoke } from "@tauri-apps/api/core"

type AddConfigFormProps = {
  onAdd: (config: PortForwardConfig) => void
  onUpdate?: (oldName: string, newConfig: PortForwardConfig) => void
  onClose: () => void
  error?: string
  onClearError: () => void
  editingConfig?: {
    config: PortForwardConfig
    index: number
  } | null
}

let AddConfigForm = ({
  onAdd,
  onUpdate,
  onClose,
  error,
  onClearError,
  editingConfig,
}: AddConfigFormProps) => {
  let [connectionType, setConnectionType] = useState<"kubernetes" | "ssh">(
    editingConfig?.config.forward_type === "Ssh" ? "ssh" : "kubernetes",
  )
  let [testStatus, setTestStatus] = useState<"idle" | "testing" | "success" | "error">("idle")
  let [testMessage, setTestMessage] = useState("")
  let [nameValue, setNameValue] = useState("")
  let [isNameManuallyChanged, setIsNameManuallyChanged] = useState(false)
  let [sshHost, setSshHost] = useState("")
  let [sshPort, setSshPort] = useState("")
  let [portsInput, setPortsInput] = useState("")
  let nameInputRef = useRef<HTMLInputElement>(null)

  let kubernetesData = useKubernetesDataFlow({
    setError: onClearError,
    editingConfig,
  })

  let formState = useFormState({
    onAdd,
    onUpdate,
    onClose,
    editingConfig,
  })

  let {
    selectedContext,
    selectedNamespace,
    selectedService,
    setSelectedContext,
    setSelectedNamespace,
    setSelectedService,
    contexts,
    namespaces,
    services,
    ports,
  } = kubernetesData

  let { handleCancel, isEditing, defaultValues } = formState
  let handleFormSubmit = formState.handleSubmit(selectedContext, selectedNamespace, selectedService)

  // Calculate current preview name for display
  let previewName = (() => {
    if (connectionType === "ssh" && sshHost && sshPort) {
      return deriveConfigName("Ssh", "", sshHost, [sshPort])
    } else if (connectionType === "kubernetes" && selectedService && portsInput) {
      let portsArray = portsInput.split(",").map(p => p.trim()).filter(p => p.length > 0)
      if (portsArray.length > 0) {
        return deriveConfigName("Kubectl", selectedService, "", portsArray)
      }
    }
    return ""
  })()

  // Initialize form state from editing config or defaults
  useEffect(() => {
    if (editingConfig) {
      setNameValue(editingConfig.config.name)
      setIsNameManuallyChanged(true)
      if (editingConfig.config.forward_type === "Ssh") {
        setSshHost(editingConfig.config.context)
        setSshPort(editingConfig.config.ports[0] || "")
      } else {
        setPortsInput(editingConfig.config.ports.join(", "))
      }
    } else {
      setNameValue("")
      setIsNameManuallyChanged(false)
      setSshHost("")
      setSshPort("")
      setPortsInput("")
    }
  }, [editingConfig])

  // Update name automatically when selections change
  useEffect(() => {
    if (!isEditing) {
      let derivedName = ""
      if (connectionType === "ssh") {
        if (sshHost && sshPort) {
          derivedName = deriveConfigName("Ssh", "", sshHost, [sshPort])
        } else if (sshHost) {
          derivedName = sshHost.split("@").pop() || sshHost
        }
      } else if (connectionType === "kubernetes") {
        if (selectedService && portsInput) {
          let portsArray = portsInput.split(",").map(p => p.trim()).filter(p => p.length > 0)
          if (portsArray.length > 0) {
            derivedName = deriveConfigName("Kubectl", selectedService, "", portsArray)
          } else {
            derivedName = selectedService
          }
        } else if (selectedService) {
          derivedName = selectedService
        } else if (selectedNamespace) {
          derivedName = selectedNamespace
        } else if (selectedContext) {
          derivedName = selectedContext
        }
      }

      // Always update name when selections change, and reset manual flag
      if (derivedName && derivedName !== nameValue) {
        setNameValue(derivedName)
        setIsNameManuallyChanged(false)
      }
    }
  }, [connectionType, selectedContext, selectedNamespace, selectedService, sshHost, sshPort, portsInput, isEditing])

  return (
    <div className="add-form-modal">
      <div className="add-form">
        <h3>
          {isEditing ? "Edit Port Forward Configuration" : "Add New Port Forward Configuration"}
        </h3>

        <ErrorBanner error={error} onClearError={onClearError} />

        <div className="form-group">
          <label>Connection Type:</label>
          <div style={{ display: "flex", gap: "10px" }}>
            <button
              type="button"
              className={connectionType === "kubernetes" ? "tab-button active" : "tab-button"}
              onClick={() => setConnectionType("kubernetes")}
            >
              Kubernetes
            </button>
            <button
              type="button"
              className={connectionType === "ssh" ? "tab-button active" : "tab-button"}
              onClick={() => setConnectionType("ssh")}
            >
              SSH
            </button>
          </div>
        </div>

        <form onSubmit={handleFormSubmit}>
          <div className="form-group">
            <label>Name (Optional):</label>
            <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
              <input
                ref={nameInputRef}
                type="text"
                name="name"
                value={nameValue}
                onChange={(e) => {
                  setNameValue(e.target.value)
                  setIsNameManuallyChanged(true)
                }}
                placeholder={previewName ? `Will be: ${previewName}` : "Auto-generated from service/host and port"}
                style={{
                  flex: 1
                }}
              />
              {(isNameManuallyChanged && !isEditing && previewName) && (
                <button
                  type="button"
                  onClick={() => {
                    setNameValue("")
                    setIsNameManuallyChanged(false)
                    if (nameInputRef.current) {
                      nameInputRef.current.value = ""
                    }
                  }}
                  style={{
                    background: "none",
                    border: "1px solid #ccc",
                    borderRadius: "4px",
                    cursor: "pointer",
                    padding: "4px 8px",
                    fontSize: "12px",
                    color: "#666",
                    minWidth: "24px",
                    height: "24px",
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center"
                  }}
                  title="Reset to auto-generated name"
                >
                  ×
                </button>
              )}
            </div>
            <small>Leave empty to auto-generate. Updates as you select service/host and ports.</small>
          </div>

          {connectionType === "kubernetes" ? (
            <>
              <KubernetesSelect
                label="Context"
                name="context"
                value={selectedContext}
                onChange={setSelectedContext}
                options={contexts.data}
                loading={contexts.loading}
                required
                loadingText="Loading contexts..."
                placeholderText="Select context..."
                editingConfig={editingConfig}
                originalValueKey="context"
              />

              <KubernetesSelect
                label="Namespace"
                name="namespace"
                value={selectedNamespace}
                onChange={setSelectedNamespace}
                options={namespaces.data}
                loading={namespaces.loading}
                disabled={!selectedContext}
                required
                loadingText="Loading namespaces..."
                placeholderText="Select namespace..."
                editingConfig={editingConfig}
                originalValueKey="namespace"
              />

              <KubernetesSelect
                label="Service"
                name="service"
                value={selectedService}
                onChange={setSelectedService}
                options={services.data}
                loading={services.loading}
                disabled={!selectedContext || !selectedNamespace}
                required
                loadingText="Loading services..."
                placeholderText="Select service..."
                editingConfig={editingConfig}
                originalValueKey="service"
              />

              <div className="form-group">
                <label>Local Interface (Optional):</label>
                <input
                  type="text"
                  name="localInterface"
                  defaultValue={defaultValues.localInterface}
                  placeholder="e.g., 127.0.0.2, 0.0.0.0"
                />
                <small>
                  Bind to specific interface (default: 127.0.0.1). Will create if doesn't exist.
                </small>
              </div>

              <div className="form-group">
                <label>Ports:</label>
                <PortSuggestions ports={ports.data} loading={ports.loading} />
                <input
                  type="text"
                  name="ports"
                  value={portsInput}
                  onChange={(e) => setPortsInput(e.target.value)}
                  placeholder="e.g., 8080:80, 9090:3000"
                  required
                />
                <small>Comma-separated list of local:remote ports</small>
              </div>
            </>
          ) : (
            <>
              <div className="form-group">
                <label>SSH Host:</label>
                <input
                  type="text"
                  name="sshHost"
                  value={sshHost}
                  onChange={(e) => setSshHost(e.target.value)}
                  placeholder="e.g., user@hostname or hostname"
                  required
                  className={
                    testStatus === "success"
                      ? "input-success"
                      : testStatus === "error"
                        ? "input-error"
                        : ""
                  }
                />
                <small>SSH connection string (user@host or just host)</small>
              </div>

              <div className="form-group">
                <label>Port:</label>
                <input
                  type="text"
                  name="sshPort"
                  value={sshPort}
                  onChange={(e) => setSshPort(e.target.value)}
                  placeholder="e.g., 8080:80"
                  required
                  className={
                    testStatus === "success"
                      ? "input-success"
                      : testStatus === "error"
                        ? "input-error"
                        : ""
                  }
                />
                <small>Port mapping in format local:remote</small>
              </div>

              <div className="form-group">
                <button
                  type="button"
                  className={`test-button ${testStatus === "testing" ? "testing" : ""}`}
                  disabled={testStatus === "testing"}
                  onClick={async () => {
                    if (!sshHost) {
                      setTestStatus("error")
                      setTestMessage("Please enter SSH host")
                      return
                    }

                    setTestStatus("testing")
                    setTestMessage("Testing SSH connection...")

                    try {
                      let result = await invoke<string>("test_ssh_connection", { sshHost })
                      setTestStatus("success")
                      setTestMessage(result)
                    } catch (error) {
                      setTestStatus("error")
                      setTestMessage(error as string)
                    }
                  }}
                >
                  {testStatus === "testing" ? "Testing..." : "Test Connection"}
                </button>
                <small>Test SSH connection before adding</small>
              </div>

              {testMessage && (
                <div className={`test-feedback ${testStatus}`}>
                  <span className="test-icon">
                    {testStatus === "success" ? "✓" : testStatus === "error" ? "✗" : "⏳"}
                  </span>
                  <span className="test-message">{testMessage}</span>
                </div>
              )}
            </>
          )}

          <input
            type="hidden"
            name="forwardType"
            value={connectionType === "ssh" ? "Ssh" : "Kubectl"}
          />

          <FormActions isEditing={isEditing} onCancel={handleCancel} />
        </form>
      </div>
    </div>
  )
}

export default AddConfigForm

