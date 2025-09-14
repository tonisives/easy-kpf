import { useState, useEffect } from "react"
import { PortForwardConfig } from "./hooks"

type UseConnectionFormProps = {
  editingConfig?: {
    config: PortForwardConfig
    index: number
  } | null
}

export let useConnectionForm = ({ editingConfig }: UseConnectionFormProps) => {
  let [connectionType, setConnectionType] = useState<"kubernetes" | "ssh">(
    editingConfig?.config.forward_type === "Ssh" ? "ssh" : "kubernetes"
  )
  let [sshHost, setSshHost] = useState("")
  let [sshPort, setSshPort] = useState("")
  let [portsInput, setPortsInput] = useState("")

  // Initialize form state from editing config or defaults
  useEffect(() => {
    if (editingConfig) {
      if (editingConfig.config.forward_type === "Ssh") {
        setSshHost(editingConfig.config.context)
        setSshPort(editingConfig.config.ports[0] || "")
      } else {
        setPortsInput(editingConfig.config.ports.join(", "))
      }
    } else {
      setSshHost("")
      setSshPort("")
      setPortsInput("")
    }
  }, [editingConfig])

  return {
    connectionType,
    setConnectionType,
    sshHost,
    setSshHost,
    sshPort,
    setSshPort,
    portsInput,
    setPortsInput,
  }
}