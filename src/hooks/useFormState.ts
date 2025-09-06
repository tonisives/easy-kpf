import { PortForwardConfig } from "./hooks"

type FormStateProps = {
  onAdd: (config: PortForwardConfig) => void
  onUpdate?: (oldName: string, newConfig: PortForwardConfig) => void
  onClose: () => void
  editingConfig?: {
    config: PortForwardConfig
    index: number
  } | null
}

export let useFormState = ({ onAdd, onUpdate, onClose, editingConfig }: FormStateProps) => {
  let handleSubmit = (selectedContext: string, selectedNamespace: string, selectedService: string) => (e: React.FormEvent) => {
    e.preventDefault()
    let formData = new FormData(e.target as HTMLFormElement)
    let portsString = formData.get("ports") as string
    let ports = portsString
      .split(",")
      .map((p) => p.trim())
      .filter((p) => p.length > 0)

    let config: PortForwardConfig = {
      name: formData.get("name") as string,
      context: selectedContext,
      namespace: selectedNamespace,
      service: selectedService,
      ports: ports,
    }

    if (editingConfig && onUpdate) {
      onUpdate(editingConfig.config.name, config)
    } else {
      onAdd(config)
    }
    onClose()
  }

  let handleCancel = () => {
    onClose()
  }

  return {
    handleSubmit,
    handleCancel,
    isEditing: !!editingConfig,
    defaultValues: {
      name: editingConfig?.config.name || "",
      ports: editingConfig?.config.ports.join(", ") || "",
    },
  }
}