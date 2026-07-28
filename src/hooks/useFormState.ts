import { PortForwardConfig, RecoveryHook, RecoverySettings } from "./hooks"

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

let formInteger = (formData: FormData, name: string, fallback: number, minimum = 0) => {
  let parsed = Number(formData.get(name))
  return Number.isFinite(parsed) ? Math.max(minimum, Math.trunc(parsed)) : fallback
}

export let parseRecovery = (
  formData: FormData,
  existing?: RecoverySettings,
): RecoverySettings | undefined => {
  if (!formData.has("recoveryOverride")) return undefined

  let hookType: RecoveryHook["type"] =
    formData.get("recoveryHookType") === "ssh" ? "ssh" : "command"
  let command = String(formData.get("recoveryHookCommand") || "").trim()
  let hook: RecoveryHook | undefined = command
    ? {
        type: hookType,
        command,
        args: String(formData.get("recoveryHookArgs") || "")
          .split("\n")
          .map((value) => value.trim())
          .filter(Boolean),
        ssh_host: hookType === "ssh"
          ? String(formData.get("recoveryHookSshHost") || "").trim() || undefined
          : undefined,
        timeout_seconds: formInteger(formData, "recoveryHookTimeoutSeconds", 30, 1),
        cooldown_seconds: formInteger(formData, "recoveryHookCooldownSeconds", 30),
      }
    : undefined

  return {
    reconnect: {
      enabled: formData.has("recoveryEnabled"),
      initial_delay_ms: formInteger(formData, "recoveryInitialDelayMs", 1000),
      max_delay_ms: formInteger(formData, "recoveryMaxDelayMs", 30000),
      stable_after_seconds: formInteger(formData, "recoveryStableAfterSeconds", 30),
      max_attempts: formInteger(formData, "recoveryMaxAttempts", 0),
    },
    hooks: {
      on_failure: existing?.hooks.on_failure || [],
      before_reconnect: [
        ...(hook ? [hook] : []),
        ...(existing?.hooks.before_reconnect.slice(1) || []),
      ],
      on_recovered: existing?.hooks.on_recovered || [],
    },
  }
}

export let useFormState = ({ onAdd, onUpdate, onClose, editingConfig }: FormStateProps) => {
  let handleSubmit = (selectedContext: string, selectedNamespace: string, selectedService: string) => (e: React.FormEvent) => {
    e.preventDefault()
    let formData = new FormData(e.target as HTMLFormElement)
    let forwardType = formData.get("forwardType") as "Kubectl" | "Ssh"
    let providedName = formData.get("name") as string
    let recovery = parseRecovery(formData, editingConfig?.config.recovery)

    let config: PortForwardConfig

    if (forwardType === "Ssh") {
      let sshHost = formData.get("sshHost") as string
      let sshPort = formData.get("sshPort") as string
      let localInterface = formData.get("localInterface") as string
      let ports = [sshPort]

      let derivedName = providedName || deriveConfigName(forwardType, selectedService, sshHost, ports)

      config = {
        name: derivedName,
        context: sshHost,
        namespace: "default",
        service: sshHost,
        ports: ports,
        local_interface: localInterface || undefined,
        forward_type: "Ssh",
        recovery,
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
        recovery,
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
