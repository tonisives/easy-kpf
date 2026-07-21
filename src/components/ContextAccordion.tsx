import { SortableContext, useSortable, verticalListSortingStrategy } from "@dnd-kit/sortable"
import { CSS } from "@dnd-kit/utilities"
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
  isExpanded: boolean
  onToggle: () => void
  dragDisabled?: boolean
}

let ContextAccordion = ({
  group,
  services,
  loading,
  onStart,
  onStop,
  onSettings,
  onClearError,
  isExpanded,
  onToggle,
  dragDisabled = false,
}: ContextAccordionProps) => {
  let groupId = `group:${group.context}`
  let {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({
    id: groupId,
    data: { type: "group", groupKey: group.context },
    disabled: dragDisabled,
  })

  let style = {
    transform: CSS.Transform.toString(transform),
    transition: isDragging ? "none" : transition || "transform 200ms ease",
    opacity: isDragging ? 0.5 : 1,
  }

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
    <div
      ref={setNodeRef}
      style={style}
      className={`context-accordion ${isDragging ? "is-dragging" : ""}`}
    >
      <div className="accordion-header">
        <button
          type="button"
          className="accordion-toggle"
          onClick={onToggle}
          aria-expanded={isExpanded}
        >
          <span className="accordion-title">
            <span className={`expand-icon ${isExpanded ? "expanded" : ""}`}>
              <svg width="8" height="11" viewBox="0 0 8 11" aria-hidden="true">
                <path d="m2 1.5 4 4-4 4" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
              </svg>
            </span>
            <h3>{getContextDisplayName(group.context, group.configs)}</h3>
            <span className="config-count">
              {group.configs.length} service{group.configs.length !== 1 ? 's' : ''} · {runningCount} connected
            </span>
          </span>
        </button>
        <button
          type="button"
          className="group-drag-handle"
          title={dragDisabled ? "Clear the filter to reorder groups" : "Drag to reorder group"}
          disabled={dragDisabled}
          {...attributes}
          {...listeners}
        >
          <svg width="10" height="16" viewBox="0 0 10 16" aria-hidden="true">
            <g fill="currentColor"><circle cx="3" cy="4" r="1"/><circle cx="7" cy="4" r="1"/><circle cx="3" cy="8" r="1"/><circle cx="7" cy="8" r="1"/><circle cx="3" cy="12" r="1"/><circle cx="7" cy="12" r="1"/></g>
          </svg>
        </button>
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
