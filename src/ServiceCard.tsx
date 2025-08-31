type ServiceCardProps = {
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
  draggable?: boolean
  onDragStart?: (e: React.DragEvent<HTMLDivElement>) => void
  onDragOver?: (e: React.DragEvent<HTMLDivElement>) => void
  onDrop?: (e: React.DragEvent<HTMLDivElement>) => void
  isDragOver?: boolean
}

function ServiceCard({
  displayName,
  context,
  namespace,
  ports,
  isRunning,
  isLoading,
  onStart,
  onStop,
  onSettings,
  draggable = false,
  onDragStart,
  onDragOver,
  onDrop,
  isDragOver = false,
}: ServiceCardProps) {
  return (
    <div 
      className={`service-group ${isDragOver ? 'drag-over' : ''}`}
      draggable={draggable}
      onDragStart={onDragStart}
      onDragOver={onDragOver}
      onDrop={onDrop}
    >
      <div className="service-header">
        {draggable && (
          <div className="drag-handle" title="Drag to reorder">
            ⋮⋮
          </div>
        )}
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

