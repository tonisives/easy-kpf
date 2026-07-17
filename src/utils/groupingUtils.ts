import type { PortForwardConfig } from "../hooks/hooks"

export type GroupedConfig = {
  context: string
  configs: PortForwardConfig[]
}

export let getConfigGroupKey = (config: PortForwardConfig) =>
  config.forward_type === "Ssh" ? "SSH" : config.context

export let groupConfigsByContext = (configs: PortForwardConfig[]): GroupedConfig[] => {
  let grouped = configs.reduce((acc, config) => {
    let contextKey = getConfigGroupKey(config)

    if (!acc[contextKey]) {
      acc[contextKey] = []
    }
    acc[contextKey].push(config)
    return acc
  }, {} as Record<string, PortForwardConfig[]>)

  return Object.entries(grouped).map(([context, configs]) => ({
    context,
    configs
  }))
}
