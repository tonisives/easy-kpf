import { useSortable } from '@dnd-kit/sortable'
import { CSS } from '@dnd-kit/utilities'

type ServiceCardProps = {
  id: string
  name: string
  displayName: string
  context: string
  namespace: string
  ports: string
  isRunning: boolean
  isLoading: boolean
  error?: string
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
  error,
  onStart,
  onStop,
  onSettings,
  onClearError,
}: ServiceCardProps) => {
  let {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({ id })

  let style = {
    transform: CSS.Transform.toString(transform),
    transition: isDragging ? 'none' : transition || 'transform 200ms ease',
    opacity: isDragging ? 0.5 : 1,
  }

  return (
    <div
      ref={setNodeRef}
      style={style}
      className="service-group"
    >
      <div className="service-header">
        <div
          className="drag-handle"
          title="Drag to reorder"
          {...attributes}
          {...listeners}
        >
          ⋮⋮
        </div>
        <div className="service-info">
          <div style={{ display: "flex", gap: "10px" }}>
            <h3>{displayName}</h3>

            <span className={`status ${isRunning ? "running" : "stopped"}`}>
              {isRunning ? "● Running" : "● Stopped"}
            </span>
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
            ⚙️
          </button>
        </div>
      </div>
      
      {error && (
        <div className="service-error">
          <span className="service-error-text">{error}</span>
          <button 
            onClick={onClearError} 
            className="service-error-close"
            title="Clear error"
          >
            ×
          </button>
        </div>
      )}
    </div>
  )
}

export default ServiceCard

