import { PortForwardConfig } from "../hooks/hooks"
import { useKubernetesDataFlow } from "../hooks/useKubernetesDataFlow"
import { useFormState } from "../hooks/useFormState"
import { KubernetesSelect } from "./KubernetesSelect"
import { PortSuggestions } from "./PortSuggestions"
import { ErrorBanner } from "./ErrorBanner"
import { FormActions } from "./FormActions"
import { useState } from "react"
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
    editingConfig?.config.forward_type === "Ssh" ? "ssh" : "kubernetes"
  )
  let [testStatus, setTestStatus] = useState<"idle" | "testing" | "success" | "error">("idle")
  let [testMessage, setTestMessage] = useState("")

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

  return (
    <div className="add-form-modal">
      <div className="add-form">
        <h3>{isEditing ? "Edit Port Forward Configuration" : "Add New Port Forward Configuration"}</h3>
        
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
            <label>Name:</label>
            <input
              type="text"
              name="name"
              defaultValue={defaultValues.name}
              placeholder="Custom name for this configuration"
              required
            />
            <small>A friendly name to identify this port forward</small>
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
                <small>Bind to specific interface (default: 127.0.0.1). Will create if doesn't exist.</small>
              </div>

              <div className="form-group">
                <label>Ports:</label>
                <PortSuggestions ports={ports.data} loading={ports.loading} />
                <input 
                  type="text" 
                  name="ports" 
                  defaultValue={defaultValues.ports}
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
                  defaultValue={defaultValues.sshHost}
                  placeholder="e.g., user@hostname or hostname"
                  required
                  className={testStatus === "success" ? "input-success" : testStatus === "error" ? "input-error" : ""}
                />
                <small>SSH connection string (user@host or just host)</small>
              </div>

              <div className="form-group">
                <label>Port:</label>
                <input
                  type="text"
                  name="sshPort"
                  defaultValue={defaultValues.sshPort}
                  placeholder="e.g., 8080:80"
                  required
                  className={testStatus === "success" ? "input-success" : testStatus === "error" ? "input-error" : ""}
                />
                <small>Port mapping in format local:remote</small>
              </div>

              <div className="form-group">
                <button
                  type="button"
                  className={`test-button ${testStatus === "testing" ? "testing" : ""}`}
                  disabled={testStatus === "testing"}
                  onClick={async () => {
                    let formData = new FormData(document.querySelector('form') as HTMLFormElement)
                    let sshHost = formData.get("sshHost") as string
                    
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

          <input type="hidden" name="forwardType" value={connectionType === "ssh" ? "Ssh" : "Kubectl"} />

          <FormActions isEditing={isEditing} onCancel={handleCancel} />
        </form>
      </div>
    </div>
  )
}

export default AddConfigForm