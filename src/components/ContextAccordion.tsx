import { useState } from "react"
import { SortableContext, verticalListSortingStrategy } from "@dnd-kit/sortable"
import ServiceCard from "../ServiceCard"
import { PortForwardConfig, ServiceStatus } from "../hooks/hooks"
import { GroupedConfig } from "../utils/groupingUtils"

type ContextAccordionProps = {
  group: GroupedConfig
  services: ServiceStatus[]
  loading: string | null
  onStart: (serviceName: string) => void
  onStop: (serviceName: string) => void
  onSettings: (serviceName: string) => void
  onClearError: (serviceName: string) => void
}

let ContextAccordion = ({
  group,
  services,
  loading,
  onStart,
  onStop,
  onSettings,
  onClearError,
}: ContextAccordionProps) => {
  let [isExpanded, setIsExpanded] = useState(true)

  let toggleExpanded = () => setIsExpanded(!isExpanded)

  let getContextDisplayName = (context: string, configs: PortForwardConfig[]) => {
    if (configs.some(config => config.forward_type === "Ssh")) {
      return "SSH"
    }
    return context
  }

  let runningCount = group.configs.filter(config =>
    services.find(s => s.name === config.name)?.running
  ).length

  return (
    <div className="context-accordion">
      <div className="accordion-header" onClick={toggleExpanded}>
        <div className="accordion-title">
          <span className="expand-icon" style={{
            transform: isExpanded ? "rotate(90deg)" : "rotate(0deg)",
            transition: "transform 0.2s"
          }}>
            ▶
          </span>
          <h3>{getContextDisplayName(group.context, group.configs)}</h3>
          <span className="config-count">
            ({group.configs.length} service{group.configs.length !== 1 ? 's' : ''}, {runningCount} running)
          </span>
        </div>
      </div>

      {isExpanded && (
        <div className="accordion-content">
          <SortableContext
            items={group.configs.map((config) => config.name)}
            strategy={verticalListSortingStrategy}
          >
            {group.configs.map((config) => {
              let service = services.find((s) => s.name === config.name)

              let displayInfo = config.forward_type === "Ssh"
                ? {
                    displayName: config.name,
                    context: config.service,
                    namespace: config.forward_type,
                    ports: `Ports: ${config.ports.join(", ")}`
                  }
                : {
                    displayName: `${config.name} (${config.service})`,
                    context: config.context,
                    namespace: config.namespace,
                    ports: `Ports: ${config.ports.join(", ")}`
                  }

              return (
                <ServiceCard
                  key={config.name}
                  id={config.name}
                  name={config.name}
                  displayName={displayInfo.displayName}
                  context={displayInfo.context}
                  namespace={displayInfo.namespace}
                  ports={displayInfo.ports}
                  isRunning={service?.running || false}
                  isLoading={loading === config.name}
                  errors={service?.errors}
                  onStart={() => onStart(config.name)}
                  onStop={() => onStop(config.name)}
                  onSettings={() => onSettings(config.name)}
                  onClearError={() => onClearError(config.name)}
                />
              )
            })}
          </SortableContext>
        </div>
      )}
    </div>
  )
}

export default ContextAccordion