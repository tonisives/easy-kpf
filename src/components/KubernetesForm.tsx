import { KubernetesSelect } from "./KubernetesSelect"
import { PortSuggestions } from "./PortSuggestions"
import { PortForwardConfig } from "../hooks/hooks"

type KubernetesFormProps = {
  selectedContext: string
  selectedNamespace: string
  selectedService: string
  portsInput: string
  contexts: { data: string[], loading: boolean }
  namespaces: { data: string[], loading: boolean }
  services: { data: string[], loading: boolean }
  ports: { data: string[], loading: boolean }
  defaultLocalInterface: string
  editingConfig?: {
    config: PortForwardConfig
    index: number
  } | null
  onContextChange: (context: string) => void
  onNamespaceChange: (namespace: string) => void
  onServiceChange: (service: string) => void
  onPortsChange: (ports: string) => void
}

export let KubernetesForm = ({
  selectedContext,
  selectedNamespace,
  selectedService,
  portsInput,
  contexts,
  namespaces,
  services,
  ports,
  defaultLocalInterface,
  editingConfig,
  onContextChange,
  onNamespaceChange,
  onServiceChange,
  onPortsChange,
}: KubernetesFormProps) => {
  return (
    <>
      <KubernetesSelect
        label="Context"
        name="context"
        value={selectedContext}
        onChange={onContextChange}
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
        onChange={onNamespaceChange}
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
        onChange={onServiceChange}
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
          defaultValue={defaultLocalInterface}
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
          onChange={(e) => onPortsChange(e.target.value)}
          placeholder="e.g., 8080:80, 9090:3000"
          required
        />
        <small>Comma-separated list of local:remote ports</small>
      </div>
    </>
  )
}