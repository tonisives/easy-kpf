import { PortForwardConfig } from "../hooks/hooks"

export type GroupedConfig = {
  context: string
  configs: PortForwardConfig[]
}

export let groupConfigsByContext = (configs: PortForwardConfig[]): GroupedConfig[] => {
  let grouped = configs.reduce((acc, config) => {
    let contextKey = config.forward_type === "Ssh" ? "SSH" : config.context

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