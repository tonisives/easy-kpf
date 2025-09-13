import { PortForwardConfig } from "../hooks/hooks"

export let parseFormData = (formData: FormData, selectedContext: string, selectedNamespace: string, selectedService: string): PortForwardConfig => {
  let portsString = formData.get("ports") as string
  let ports = portsString
    .split(",")
    .map((p) => p.trim())
    .filter((p) => p.length > 0)

  return {
    name: formData.get("name") as string,
    context: selectedContext,
    namespace: selectedNamespace,
    service: selectedService,
    ports: ports,
    forward_type: (formData.get("forward_type") as "Kubectl" | "Ssh") || "Kubectl",
  }
}

export let createFormSubmitHandler = (
  selectedContext: string,
  selectedNamespace: string,
  selectedService: string,
  onAdd: (config: PortForwardConfig) => void,
  onUpdate: ((oldName: string, newConfig: PortForwardConfig) => void) | undefined,
  onClose: () => void,
  editingConfig?: {
    config: PortForwardConfig
    index: number
  } | null
) => (e: React.FormEvent) => {
  e.preventDefault()
  let formData = new FormData(e.target as HTMLFormElement)
  let config = parseFormData(formData, selectedContext, selectedNamespace, selectedService)

  if (editingConfig && onUpdate) {
    onUpdate(editingConfig.config.name, config)
  } else {
    onAdd(config)
  }
  onClose()
}