import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

type ServiceStatus = {
  name: string;
  running: boolean;
};

function App() {
  const [services, setServices] = useState<ServiceStatus[]>([
    { name: "db-s", running: false },
    { name: "grafana-docn", running: false }
  ]);
  const [message, setMessage] = useState("");
  const [loading, setLoading] = useState<string | null>(null);

  const updateServiceStatus = async () => {
    try {
      const runningServices: string[] = await invoke("get_running_services");
      setServices(prev => prev.map(service => ({
        ...service,
        running: runningServices.includes(service.name)
      })));
    } catch (error) {
      console.error("Failed to get running services:", error);
    }
  };

  useEffect(() => {
    updateServiceStatus();
  }, []);

  const setKubectlContext = async () => {
    setLoading("context");
    try {
      const result: string = await invoke("set_kubectl_context");
      setMessage(result);
    } catch (error) {
      setMessage(`Error: ${error}`);
    } finally {
      setLoading(null);
    }
  };

  const startDbPortForward = async () => {
    setLoading("db-s");
    try {
      const result: string = await invoke("start_db_port_forward");
      setMessage(result);
      await updateServiceStatus();
    } catch (error) {
      setMessage(`Error: ${error}`);
    } finally {
      setLoading(null);
    }
  };

  const startGrafanaPortForward = async () => {
    setLoading("grafana-docn");
    try {
      const result: string = await invoke("start_grafana_port_forward");
      setMessage(result);
      await updateServiceStatus();
    } catch (error) {
      setMessage(`Error: ${error}`);
    } finally {
      setLoading(null);
    }
  };

  const stopPortForward = async (serviceName: string) => {
    setLoading(serviceName);
    try {
      const result: string = await invoke("stop_port_forward", { serviceName });
      setMessage(result);
      await updateServiceStatus();
    } catch (error) {
      setMessage(`Error: ${error}`);
    } finally {
      setLoading(null);
    }
  };

  return (
    <main className="container">
      <h1>Kubernetes Port Forwarding</h1>
      
      <div className="context-section">
        <button 
          onClick={setKubectlContext}
          disabled={loading === "context"}
          className="context-button"
        >
          {loading === "context" ? "Setting Context..." : "Set kubectl context (hs-docn-cluster-1)"}
        </button>
      </div>

      <div className="services-section">
        <h2>Port Forwarding Services</h2>
        
        <div className="service-group">
          <div className="service-info">
            <h3>DB Service (db-s)</h3>
            <p>Ports: 5332:25060, 5333:25061, 5334:25062, 5335:25063, 5336:25064</p>
            <span className={`status ${services.find(s => s.name === "db-s")?.running ? "running" : "stopped"}`}>
              {services.find(s => s.name === "db-s")?.running ? "● Running" : "● Stopped"}
            </span>
          </div>
          <div className="service-controls">
            {!services.find(s => s.name === "db-s")?.running ? (
              <button 
                onClick={startDbPortForward}
                disabled={loading === "db-s"}
                className="start-button"
              >
                {loading === "db-s" ? "Starting..." : "Start DB Port Forward"}
              </button>
            ) : (
              <button 
                onClick={() => stopPortForward("db-s")}
                disabled={loading === "db-s"}
                className="stop-button"
              >
                {loading === "db-s" ? "Stopping..." : "Stop DB Port Forward"}
              </button>
            )}
          </div>
        </div>

        <div className="service-group">
          <div className="service-info">
            <h3>Grafana (grafana-docn)</h3>
            <p>Port: 2999:80</p>
            <span className={`status ${services.find(s => s.name === "grafana-docn")?.running ? "running" : "stopped"}`}>
              {services.find(s => s.name === "grafana-docn")?.running ? "● Running" : "● Stopped"}
            </span>
          </div>
          <div className="service-controls">
            {!services.find(s => s.name === "grafana-docn")?.running ? (
              <button 
                onClick={startGrafanaPortForward}
                disabled={loading === "grafana-docn"}
                className="start-button"
              >
                {loading === "grafana-docn" ? "Starting..." : "Start Grafana Port Forward"}
              </button>
            ) : (
              <button 
                onClick={() => stopPortForward("grafana-docn")}
                disabled={loading === "grafana-docn"}
                className="stop-button"
              >
                {loading === "grafana-docn" ? "Stopping..." : "Stop Grafana Port Forward"}
              </button>
            )}
          </div>
        </div>
      </div>

      {message && (
        <div className="message">
          <pre>{message}</pre>
          <button onClick={() => setMessage("")} className="clear-button">Clear</button>
        </div>
      )}
    </main>
  );
}

export default App;
