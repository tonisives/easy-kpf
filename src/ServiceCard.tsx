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
  onStart: () => void
  onStop: () => void
  onSettings: () => void
}

let ServiceCard = ({
  id,
  displayName,
  context,
  namespace,
  ports,
  isRunning,
  isLoading,
  onStart,
  onStop,
  onSettings,
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
    transition,
    opacity: isDragging ? 0.3 : 1,
    zIndex: isDragging ? 1000 : 'auto',
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
    </div>
  )
}

export default ServiceCard

