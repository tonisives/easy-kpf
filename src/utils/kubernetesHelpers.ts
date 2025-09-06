import { invoke } from "@tauri-apps/api/core"

export let loadKubernetesContexts = (): Promise<string[]> => 
  invoke("get_kubectl_contexts")

export let loadKubernetesNamespaces = (context: string): Promise<string[]> => 
  invoke("get_namespaces", { context })

export let loadKubernetesServices = (context: string, namespace: string): Promise<string[]> => 
  invoke("get_services", { context, namespace })

export let loadKubernetesPorts = (context: string, namespace: string, service: string): Promise<string[]> => 
  invoke("get_service_ports", { context, namespace, service })