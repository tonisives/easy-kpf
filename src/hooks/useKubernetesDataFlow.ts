import { useState, useEffect } from "react"
import { useAsyncLoader } from "./useAsyncLoader"
import { PortForwardConfig } from "./hooks"
import {
  loadKubernetesContexts,
  loadKubernetesNamespaces,
  loadKubernetesServices,
  loadKubernetesPorts,
} from "../utils/kubernetesHelpers"

type KubernetesDataFlowProps = {
  setError?: (error: string) => void
  editingConfig?: {
    config: PortForwardConfig
    index: number
  } | null
}

export let useKubernetesDataFlow = ({
  setError,
  editingConfig,
}: KubernetesDataFlowProps) => {
  let [selectedContext, setSelectedContext] = useState(editingConfig?.config.context || "")
  let [selectedNamespace, setSelectedNamespace] = useState(editingConfig?.config.namespace || "")
  let [selectedService, setSelectedService] = useState(editingConfig?.config.service || "")

  let contexts = useAsyncLoader(loadKubernetesContexts, setError)
  let namespaces = useAsyncLoader(
    (context: string) => loadKubernetesNamespaces(context),
    setError
  )
  let services = useAsyncLoader(
    (context: string, namespace: string) => loadKubernetesServices(context, namespace),
    setError
  )
  let ports = useAsyncLoader(
    (context: string, namespace: string, service: string) =>
      loadKubernetesPorts(context, namespace, service),
    setError
  )

  // Auto-load contexts on mount
  useEffect(() => {
    contexts.load()
  }, [])

  // Auto-load namespaces when context changes or for editing mode
  useEffect(() => {
    if (selectedContext && contexts.data.length > 0) {
      if (!editingConfig) {
        setSelectedNamespace("")
        setSelectedService("")
      }
      namespaces.load(selectedContext)
    }
  }, [selectedContext, contexts.data.length])

  // Auto-select first available namespace when namespaces load
  useEffect(() => {
    if (namespaces.data.length > 0 && !selectedNamespace && !namespaces.loading) {
      setSelectedNamespace(namespaces.data[0])
    }
  }, [namespaces.data, namespaces.loading])

  // Auto-load services when namespace changes or for editing mode
  useEffect(() => {
    if (selectedContext && selectedNamespace && namespaces.data.length > 0) {
      if (!editingConfig) {
        setSelectedService("")
      }
      services.load(selectedContext, selectedNamespace)
    }
  }, [selectedNamespace, namespaces.data.length])

  // Auto-select first available service when services load
  useEffect(() => {
    if (services.data.length > 0 && !selectedService && !services.loading) {
      setSelectedService(services.data[0])
    }
  }, [services.data, services.loading])

  // Auto-load ports when service changes
  useEffect(() => {
    if (selectedContext && selectedNamespace && selectedService && services.data.length > 0) {
      ports.load(selectedContext, selectedNamespace, selectedService)
    }
  }, [selectedService, services.data.length])

  return {
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
  }
}