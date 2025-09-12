import { PortForwardConfig } from "../hooks/hooks"
import { useKubernetesDataFlow } from "../hooks/useKubernetesDataFlow"
import { useFormState } from "../hooks/useFormState"
import { KubernetesSelect } from "./KubernetesSelect"
import { PortSuggestions } from "./PortSuggestions"
import { ErrorBanner } from "./ErrorBanner"
import { FormActions } from "./FormActions"

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
            <label>Forward Type:</label>
            <select 
              name="forwardType" 
              defaultValue={defaultValues.forwardType}
              required
            >
              <option value="Kubectl">Kubectl Port Forward</option>
              <option value="Ssh">SSH Port Forward</option>
            </select>
            <small>Choose between kubectl or SSH port forwarding</small>
          </div>

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

          <FormActions isEditing={isEditing} onCancel={handleCancel} />
        </form>
      </div>
    </div>
  )
}

export default AddConfigForm