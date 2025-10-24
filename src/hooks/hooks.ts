import { invoke } from "@tauri-apps/api/core"
import { useEffect, useState } from "react"

export type ServiceStatus = {
  name: string
  running: boolean
  error?: string
}

export type ForwardType = "Kubectl" | "Ssh"

export type PortForwardConfig = {
  name: string
  context: string
  namespace: string
  service: string
  ports: string[]
  local_interface?: string
  forward_type: ForwardType
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
  let [formError, setFormError] = useState<string | undefined>(undefined)

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

  let syncWithExistingProcesses = async () => {
    try {
      let syncedServices: string[] = await invoke("sync_with_existing_processes")
      if (syncedServices.length > 0) {
        console.log("Synced with existing port forwards:", syncedServices)
      }
    } catch (error) {
      console.error("Failed to sync with existing processes:", error)
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

  let verifyPortForwards = async () => {
    try {
      let stoppedServices: string[] = await invoke("verify_and_update_port_forwards")
      if (stoppedServices.length > 0) {
        setServices((prev) =>
          prev.map((service) => ({
            ...service,
            running: !stoppedServices.includes(service.name) && service.running,
            error: stoppedServices.includes(service.name) 
              ? "Port forward stopped unexpectedly" 
              : service.error,
          })),
        )
      }
    } catch (error) {
      console.error("Failed to verify port forwards:", error)
    }
  }

  useEffect(() => {
    loadConfigs()
      .then(syncWithExistingProcesses)
      .then(updateServiceStatus)
    
    // Set up periodic verification of port forwards
    let verificationInterval = setInterval(() => {
      verifyPortForwards()
    }, 5000) // Check every 5 seconds
    
    return () => {
      clearInterval(verificationInterval)
    }
  }, [])

  let startPortForward = async (serviceKey: string) => {
    setLoading(serviceKey)
    setServices((prev) =>
      prev.map((service) =>
        service.name === serviceKey ? { ...service, error: undefined } : service,
      ),
    )
    try {
      await invoke("start_port_forward_by_key", { serviceKey })
      await updateServiceStatus()
    } catch (error) {
      let errorMessage = `${error}`

      if (errorMessage.includes("port forwarding is already running")) {
        setServices((prev) =>
          prev.map((service) =>
            service.name === serviceKey ? { ...service, running: true, error: undefined } : service,
          ),
        )
      } else {
        setServices((prev) =>
          prev.map((service) =>
            service.name === serviceKey ? { ...service, error: errorMessage } : service,
          ),
        )
      }
    } finally {
      setLoading(null)
    }
  }

  let addConfig = async (config: PortForwardConfig) => {
    setFormError(undefined)
    try {
      await invoke("add_port_forward_config", { config })
      await loadConfigs()
      setMessage(`Added configuration for ${config.name}`)
    } catch (error) {
      setFormError(`Error adding config: ${error}`)
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
      await invoke("update_port_forward_config", { 
        oldServiceKey: oldServiceKey, 
        newConfig: newConfig 
      })
      await loadConfigs()
      await updateServiceStatus()
      setMessage(`Updated configuration for ${newConfig.name}`)
    } catch (error) {
      setMessage(`Error updating config: ${error}`)
    }
  }

  let reorderConfig = async (serviceKey: string, newIndex: number) => {
    let oldIndex = configs.findIndex((config) => config.name === serviceKey)
    if (oldIndex === -1) return

    let newConfigs = [...configs]
    let [movedConfig] = newConfigs.splice(oldIndex, 1)
    newConfigs.splice(newIndex, 0, movedConfig)
    setConfigs(newConfigs)

    try {
      await invoke("reorder_port_forward_config", { serviceKey, newIndex })
    } catch (error) {
      setConfigs(configs)
      setMessage(`Error reordering config: ${error}`)
    }
  }

  let loadContexts = async () => {
    setFormError(undefined)
    try {
      let contexts: string[] = await invoke("get_kubectl_contexts")
      setAvailableContexts(contexts)
      setMessage("")
    } catch (error) {
      console.error("Failed to load contexts:", error)
      setFormError(`${error}`)
    }
  }

  let loadNamespaces = async (context: string) => {
    if (!context) return
    setFormError(undefined)
    try {
      let namespaces: string[] = await invoke("get_namespaces", { context })
      setAvailableNamespaces(namespaces)
      setMessage("")
    } catch (error) {
      console.error("Failed to load namespaces:", error)
      setFormError(`${error}`)
      setAvailableNamespaces([])
    }
  }

  let loadServices = async (context: string, namespace: string) => {
    if (!context || !namespace) return
    setFormError(undefined)
    try {
      let services: string[] = await invoke("get_services", { context, namespace })
      setAvailableServices(services)
      setMessage("")
    } catch (error) {
      console.error("Failed to load services:", error)
      setFormError(`${error}`)
      setAvailableServices([])
    }
  }

  let loadPorts = async (context: string, namespace: string, service: string) => {
    if (!context || !namespace || !service) return
    setFormError(undefined)
    try {
      let ports: string[] = await invoke("get_service_ports", { context, namespace, service })
      setAvailablePorts(ports)
      setMessage("")
    } catch (error) {
      console.error("Failed to load ports:", error)
      setFormError(`${error}`)
      setAvailablePorts([])
    }
  }

  let stopPortForward = async (serviceName: string) => {
    setLoading(serviceName)
    setServices((prev) =>
      prev.map((service) =>
        service.name === serviceName ? { ...service, error: undefined } : service,
      ),
    )
    try {
      await invoke("stop_port_forward", { serviceName })
      await updateServiceStatus()
    } catch (error) {
      let errorMessage = `${error}`

      if (errorMessage.includes("port forwarding is not running")) {
        setServices((prev) =>
          prev.map((service) =>
            service.name === serviceName
              ? { ...service, running: false, error: undefined }
              : service,
          ),
        )
      } else {
        setServices((prev) =>
          prev.map((service) =>
            service.name === serviceName ? { ...service, error: errorMessage } : service,
          ),
        )
      }
    } finally {
      setLoading(null)
    }
  }

  let clearServiceError = (serviceName: string) => {
    setServices((prev) =>
      prev.map((service) =>
        service.name === serviceName ? { ...service, error: undefined } : service,
      ),
    )
  }

  let clearFormError = () => {
    setFormError(undefined)
  }

  let reconnectAll = async () => {
    let disconnectedServices = services.filter((service) => {
      let config = configs.find((c) => c.name === service.name)
      return config && !service.running && service.error
    })

    if (disconnectedServices.length === 0) {
      return
    }

    for (let service of disconnectedServices) {
      await startPortForward(service.name)
    }
  }

  return {
    configs,
    services,
    loading,
    formError,
    loadConfigs,
    updateServiceStatus,
    startPortForward,
    addConfig,
    removeConfig,
    updateConfig,
    reorderConfig,
    loadContexts,
    loadNamespaces,
    loadServices,
    loadPorts,
    stopPortForward,
    clearServiceError,
    clearFormError,
    reconnectAll,
  }
}
