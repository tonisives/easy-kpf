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
    let forwardType = formData.get("forwardType") as "Kubectl" | "Ssh"
    
    let config: PortForwardConfig

    if (forwardType === "Ssh") {
      let sshHost = formData.get("sshHost") as string
      let sshPort = formData.get("sshPort") as string
      
      config = {
        name: formData.get("name") as string,
        context: sshHost, // Use SSH host as context for SSH mode
        namespace: "default", // Default namespace for SSH
        service: sshHost, // Use SSH host as service name
        ports: [sshPort],
        local_interface: undefined,
        forward_type: "Ssh",
      }
    } else {
      let portsString = formData.get("ports") as string
      let ports = portsString
        .split(",")
        .map((p) => p.trim())
        .filter((p) => p.length > 0)

      let localInterface = formData.get("localInterface") as string

      config = {
        name: formData.get("name") as string,
        context: selectedContext,
        namespace: selectedNamespace,
        service: selectedService,
        ports: ports,
        local_interface: localInterface || undefined,
        forward_type: "Kubectl",
      }
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
      localInterface: editingConfig?.config.local_interface || "",
      forwardType: editingConfig?.config.forward_type || "Kubectl",
      sshHost: editingConfig?.config.forward_type === "Ssh" ? editingConfig?.config.context : "",
      sshPort: editingConfig?.config.forward_type === "Ssh" ? editingConfig?.config.ports[0] : "",
    },
  }
}