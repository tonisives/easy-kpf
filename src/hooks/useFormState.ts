import { PortForwardConfig } from "./hooks"

export let deriveConfigName = (
  forwardType: "Kubectl" | "Ssh",
  selectedService: string,
  sshHost: string,
  ports: string[]
): string => {
  if (forwardType === "Ssh") {
    let host = sshHost.split("@").pop() || sshHost
    let port = ports[0]?.split(":")[0] || "unknown"
    return `${host}-${port}`
  } else {
    let port = ports[0]?.split(":")[0] || "unknown"
    return `${selectedService}-${port}`
  }
}

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
    let providedName = formData.get("name") as string

    let config: PortForwardConfig

    if (forwardType === "Ssh") {
      let sshHost = formData.get("sshHost") as string
      let sshPort = formData.get("sshPort") as string
      let ports = [sshPort]

      let derivedName = providedName || deriveConfigName(forwardType, selectedService, sshHost, ports)

      config = {
        name: derivedName,
        context: sshHost,
        namespace: "default",
        service: sshHost,
        ports: ports,
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
      let derivedName = providedName || deriveConfigName(forwardType, selectedService, "", ports)

      config = {
        name: derivedName,
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