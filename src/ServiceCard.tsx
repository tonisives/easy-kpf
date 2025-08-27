
type ServiceCardProps = {
  name: string;
  displayName: string;
  context: string;
  namespace: string;
  ports: string;
  isRunning: boolean;
  isLoading: boolean;
  onStart: () => void;
  onStop: () => void;
};

function ServiceCard({
  displayName,
  context,
  namespace,
  ports,
  isRunning,
  isLoading,
  onStart,
  onStop,
}: ServiceCardProps) {
  return (
    <div className="service-group">
      <div className="service-header">
        <div className="service-info">
          <h3>{displayName}</h3>
          <p>Context: {context} | Namespace: {namespace} | {ports}</p>
        </div>
        <div className="service-status-controls">
          <span className={`status ${isRunning ? "running" : "stopped"}`}>
            {isRunning ? "● Running" : "● Stopped"}
          </span>
          {!isRunning ? (
            <button
              onClick={onStart}
              disabled={isLoading}
              className="start-button"
            >
              {isLoading ? "Starting..." : "Start"}
            </button>
          ) : (
            <button
              onClick={onStop}
              disabled={isLoading}
              className="stop-button"
            >
              {isLoading ? "Stopping..." : "Stop"}
            </button>
          )}
        </div>
      </div>
    </div>
  );
}

export default ServiceCard;