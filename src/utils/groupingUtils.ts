import type { PortForwardConfig, RecoveryScopeKind } from "../hooks/hooks"

export type GroupedConfig = {
  key: string
  context: string
  scopeKind: RecoveryScopeKind
  configs: PortForwardConfig[]
}

export let getConfigGroupKey = (config: PortForwardConfig) =>
  config.forward_type === "Ssh"
    ? `ssh:${config.context}`
    : `kubernetes:${config.context}`

export let groupConfigsByContext = (configs: PortForwardConfig[]): GroupedConfig[] => {
  let grouped = configs.reduce((acc, config) => {
    let key = getConfigGroupKey(config)

    if (!acc[key]) {
      acc[key] = {
        key,
        context: config.context,
        scopeKind: config.forward_type === "Ssh" ? "ssh_host" : "kubernetes_context",
        configs: [],
      }
    }
    acc[key].configs.push(config)
    return acc
  }, {} as Record<string, GroupedConfig>)

  return Object.values(grouped)
}
