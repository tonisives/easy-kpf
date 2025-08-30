import { invoke } from "@tauri-apps/api/core"
import { useEffect, useState } from "react"

export type ServiceStatus = {
  name: string
  running: boolean
}

export type PortForwardConfig = {
  name: string
  context: string
  namespace: string
  service: string
  ports: string[]
}

export let useConfigs = (
  setMessage: (msg: string) => void,
  setAvailablePorts: (ports: string[]) => void,
  setAvailableContexts: (ports: string[]) => void,
  setAvailableNamespaces: (ports: string[]) => void,
  setAvailableServices: (ports: string[]) => void,
) => {
  let [configs, setConfigs] = useState<PortForwardConfig[]>([])
  let [services, setServices] = useState<ServiceStatus[]>([])
  let [loading, setLoading] = useState<string | null>(null)

  let loadConfigs = async () => {
    try {
      let loadedConfigs: PortForwardConfig[] = await invoke("get_port_forward_configs")
      setConfigs(loadedConfigs)
      setServices(loadedConfigs.map((config) => ({ name: config.name, running: false })))
    } catch (error) {
      console.error("Failed to load configs:", error)
      setMessage(`Error loading configs: ${error}`)
    }
  }

  let updateServiceStatus = async () => {
    try {
      let runningServices: string[] = await invoke("get_running_services")
      setServices((prev) =>
        prev.map((service) => ({
          ...service,
          running: runningServices.includes(service.name),
        })),
      )
    } catch (error) {
      console.error("Failed to get running services:", error)
    }
  }

  useEffect(() => {
    loadConfigs().then(updateServiceStatus)
  }, [])

  let startPortForward = async (serviceKey: string) => {
    setLoading(serviceKey)
    try {
      let result: string = await invoke("start_port_forward_by_key", { serviceKey })
      setMessage(result)
      await updateServiceStatus()
    } catch (error) {
      let errorMessage = `${error}`
      setMessage(`Error: ${errorMessage}`)

      // If error indicates port forwarding is already running, set the service state to running
      if (errorMessage.includes("port forwarding is already running")) {
        setServices((prev) =>
          prev.map((service) =>
            service.name === serviceKey ? { ...service, running: true } : service,
          ),
        )
      }
    } finally {
      setLoading(null)
    }
  }

  let addConfig = async (config: PortForwardConfig) => {
    try {
      await invoke("add_port_forward_config", { config })
      await loadConfigs()
      setMessage(`Added configuration for ${config.name}`)
    } catch (error) {
      setMessage(`Error adding config: ${error}`)
    }
  }

  let removeConfig = async (serviceKey: string) => {
    try {
      await invoke("remove_port_forward_config", { serviceKey })
      await loadConfigs()
      setMessage(`Removed configuration for ${serviceKey}`)
    } catch (error) {
      setMessage(`Error removing config: ${error}`)
    }
  }

  let updateConfig = async (oldServiceKey: string, newConfig: PortForwardConfig) => {
    try {
      await invoke("remove_port_forward_config", { serviceKey: oldServiceKey })
      await invoke("add_port_forward_config", { config: newConfig })
      await loadConfigs()
      setMessage(`Updated configuration for ${newConfig.name}`)
    } catch (error) {
      setMessage(`Error updating config: ${error}`)
    }
  }

  let loadContexts = async () => {
    try {
      let contexts: string[] = await invoke("get_kubectl_contexts")
      setAvailableContexts(contexts)
    } catch (error) {
      console.error("Failed to load contexts:", error)
    }
  }

  let loadNamespaces = async (context: string) => {
    if (!context) return
    try {
      let namespaces: string[] = await invoke("get_namespaces", { context })
      setAvailableNamespaces(namespaces)
    } catch (error) {
      console.error("Failed to load namespaces:", error)
      setAvailableNamespaces([])
    }
  }

  let loadServices = async (context: string, namespace: string) => {
    if (!context || !namespace) return
    try {
      let services: string[] = await invoke("get_services", { context, namespace })
      setAvailableServices(services)
    } catch (error) {
      console.error("Failed to load services:", error)
      setAvailableServices([])
    }
  }

  let loadPorts = async (context: string, namespace: string, service: string) => {
    if (!context || !namespace || !service) return
    try {
      let ports: string[] = await invoke("get_service_ports", { context, namespace, service })
      setAvailablePorts(ports)
    } catch (error) {
      console.error("Failed to load ports:", error)
      setAvailablePorts([])
    }
  }

  let stopPortForward = async (serviceName: string) => {
    setLoading(serviceName)
    try {
      let result: string = await invoke("stop_port_forward", { serviceName })
      setMessage(result)
      await updateServiceStatus()
    } catch (error) {
      let errorMessage = `${error}`
      setMessage(`Error: ${errorMessage}`)

      // If error indicates port forwarding is not running, reset the service state to stopped
      if (errorMessage.includes("port forwarding is not running")) {
        setServices((prev) =>
          prev.map((service) =>
            service.name === serviceName ? { ...service, running: false } : service,
          ),
        )
      }
    } finally {
      setLoading(null)
    }
  }

  return {
    configs,
    services,
    loading,
    loadConfigs,
    updateServiceStatus,
    startPortForward,
    addConfig,
    removeConfig,
    updateConfig,
    loadContexts,
    loadNamespaces,
    loadServices,
    loadPorts,
    stopPortForward,
  }
}
