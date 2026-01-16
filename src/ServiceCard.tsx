import { useSortable } from "@dnd-kit/sortable"
import { CSS } from "@dnd-kit/utilities"

type ServiceCardProps = {
  id: string
  name: string
  displayName: string
  context: string
  namespace: string
  ports: string
  isRunning: boolean
  isLoading: boolean
  errors?: string[]
  onStart: () => void
  onStop: () => void
  onSettings: () => void
  onClearError: () => void
}

let ServiceCard = ({
  id,
  displayName,
  context,
  namespace,
  ports,
  isRunning,
  isLoading,
  errors,
  onStart,
  onStop,
  onSettings,
  onClearError,
}: ServiceCardProps) => {
  let { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({ id })

  let style = {
    transform: CSS.Transform.toString(transform),
    transition: isDragging ? "none" : transition || "transform 200ms ease",
    opacity: isDragging ? 0.5 : 1,
  }

  return (
    <div ref={setNodeRef} style={style} className="service-group">
      <div className="service-header">
        <div className="drag-handle" title="Drag to reorder" {...attributes} {...listeners}>
          ⋮⋮
        </div>
        <div className="service-info">
          <div style={{ display: "flex", gap: "10px" }}>
            <h3>{displayName}</h3>
          </div>
          <p>
            Context: {context} | Namespace: {namespace} | {ports}
          </p>
        </div>
        <div className="service-status-controls">
          {!isRunning ? (
            <button onClick={onStart} disabled={isLoading} className="start-button">
              {isLoading ? "Starting..." : "Start"}
            </button>
          ) : (
            <button onClick={onStop} disabled={isLoading} className="stop-button">
              {isLoading ? "Stopping..." : "Stop"}
            </button>
          )}
          <button onClick={onSettings} className="settings-icon-button" title="Settings">
            <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
              <path d="M8 4.754a3.246 3.246 0 1 0 0 6.492 3.246 3.246 0 0 0 0-6.492zM5.754 8a2.246 2.246 0 1 1 4.492 0 2.246 2.246 0 0 1-4.492 0z"/>
              <path d="M9.796 1.343c-.527-1.79-3.065-1.79-3.592 0l-.094.319a.873.873 0 0 1-1.255.52l-.292-.16c-1.64-.892-3.433.902-2.54 2.541l.159.292a.873.873 0 0 1-.52 1.255l-.319.094c-1.79.527-1.79 3.065 0 3.592l.319.094a.873.873 0 0 1 .52 1.255l-.16.292c-.892 1.64.901 3.434 2.541 2.54l.292-.159a.873.873 0 0 1 1.255.52l.094.319c.527 1.79 3.065 1.79 3.592 0l.094-.319a.873.873 0 0 1 1.255-.52l.292.16c1.64.893 3.434-.902 2.54-2.541l-.159-.292a.873.873 0 0 1 .52-1.255l.319-.094c1.79-.527 1.79-3.065 0-3.592l-.319-.094a.873.873 0 0 1-.52-1.255l.16-.292c.893-1.64-.902-3.433-2.541-2.54l-.292.159a.873.873 0 0 1-1.255-.52l-.094-.319zm-2.633.283c.246-.835 1.428-.835 1.674 0l.094.319a1.873 1.873 0 0 0 2.693 1.115l.291-.16c.764-.415 1.6.42 1.184 1.185l-.159.292a1.873 1.873 0 0 0 1.116 2.692l.318.094c.835.246.835 1.428 0 1.674l-.319.094a1.873 1.873 0 0 0-1.115 2.693l.16.291c.415.764-.42 1.6-1.185 1.184l-.291-.159a1.873 1.873 0 0 0-2.693 1.116l-.094.318c-.246.835-1.428.835-1.674 0l-.094-.319a1.873 1.873 0 0 0-2.692-1.115l-.292.16c-.764.415-1.6-.42-1.184-1.185l.159-.291A1.873 1.873 0 0 0 1.945 8.93l-.319-.094c-.835-.246-.835-1.428 0-1.674l.319-.094A1.873 1.873 0 0 0 3.06 4.377l-.16-.292c-.415-.764.42-1.6 1.185-1.184l.292.159a1.873 1.873 0 0 0 2.692-1.115l.094-.319z"/>
            </svg>
          </button>
        </div>
      </div>

      {errors && errors.length > 0 && (
        <div className="service-error">
          <div className="service-error-text">
            {errors.map((err, index) => (
              <div key={index} className="service-error-line">{err}</div>
            ))}
          </div>
          <button onClick={onClearError} className="service-error-close" title="Clear errors">
            ×
          </button>
        </div>
      )}
    </div>
  )
}

export default ServiceCard
