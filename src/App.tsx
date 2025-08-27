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
    { name: "grafana-docn", running: false },
    { name: "postgres-cluster-rw", running: false },
    { name: "vmks-grafana", running: false }
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

  const setKubectlContext = async (context: string) => {
    setLoading("context");
    try {
      const result: string = await invoke("set_kubectl_context", { context });
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

  const startPostgresClusterPortForward = async () => {
    setLoading("postgres-cluster-rw");
    try {
      const result: string = await invoke("start_postgres_cluster_port_forward");
      setMessage(result);
      await updateServiceStatus();
    } catch (error) {
      setMessage(`Error: ${error}`);
    } finally {
      setLoading(null);
    }
  };

  const startVmksGrafanaPortForward = async () => {
    setLoading("vmks-grafana");
    try {
      const result: string = await invoke("start_vmks_grafana_port_forward");
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
        <h2>Kubectl Contexts</h2>
        <div className="context-buttons">
          <button 
            onClick={() => setKubectlContext("hs-docn-cluster-1")}
            disabled={loading === "context"}
            className="context-button"
          >
            {loading === "context" ? "Setting Context..." : "hs-docn-cluster-1"}
          </button>
          <button 
            onClick={() => setKubectlContext("tgs")}
            disabled={loading === "context"}
            className="context-button"
          >
            {loading === "context" ? "Setting Context..." : "tgs"}
          </button>
          <button 
            onClick={() => setKubectlContext("hs-gcp-cluster-1")}
            disabled={loading === "context"}
            className="context-button"
          >
            {loading === "context" ? "Setting Context..." : "hs-gcp-cluster-1"}
          </button>
        </div>
      </div>

      <div className="services-section">
        <h2>Port Forwarding Services</h2>
        
        <div className="service-group">
          <div className="service-header">
            <div className="service-info">
              <h3>DB Service (db-s)</h3>
              <p>Context: hs-docn-cluster-1 | Namespace: monitoring | Ports: 5332:25060, 5333:25061, 5334:25062, 5335:25063, 5336:25064</p>
            </div>
            <div className="service-status-controls">
              <span className={`status ${services.find(s => s.name === "db-s")?.running ? "running" : "stopped"}`}>
                {services.find(s => s.name === "db-s")?.running ? "● Running" : "● Stopped"}
              </span>
              {!services.find(s => s.name === "db-s")?.running ? (
                <button 
                  onClick={startDbPortForward}
                  disabled={loading === "db-s"}
                  className="start-button"
                >
                  {loading === "db-s" ? "Starting..." : "Start"}
                </button>
              ) : (
                <button 
                  onClick={() => stopPortForward("db-s")}
                  disabled={loading === "db-s"}
                  className="stop-button"
                >
                  {loading === "db-s" ? "Stopping..." : "Stop"}
                </button>
              )}
            </div>
          </div>
        </div>

        <div className="service-group">
          <div className="service-header">
            <div className="service-info">
              <h3>Grafana (grafana-docn)</h3>
              <p>Context: hs-docn-cluster-1 | Namespace: monitoring | Port: 2999:80</p>
            </div>
            <div className="service-status-controls">
              <span className={`status ${services.find(s => s.name === "grafana-docn")?.running ? "running" : "stopped"}`}>
                {services.find(s => s.name === "grafana-docn")?.running ? "● Running" : "● Stopped"}
              </span>
              {!services.find(s => s.name === "grafana-docn")?.running ? (
                <button 
                  onClick={startGrafanaPortForward}
                  disabled={loading === "grafana-docn"}
                  className="start-button"
                >
                  {loading === "grafana-docn" ? "Starting..." : "Start"}
                </button>
              ) : (
                <button 
                  onClick={() => stopPortForward("grafana-docn")}
                  disabled={loading === "grafana-docn"}
                  className="stop-button"
                >
                  {loading === "grafana-docn" ? "Stopping..." : "Stop"}
                </button>
              )}
            </div>
          </div>
        </div>

        <div className="service-group">
          <div className="service-header">
            <div className="service-info">
              <h3>Postgres Cluster (postgres-cluster-rw)</h3>
              <p>Context: tgs | Namespace: infra | Port: 8100:5432</p>
            </div>
            <div className="service-status-controls">
              <span className={`status ${services.find(s => s.name === "postgres-cluster-rw")?.running ? "running" : "stopped"}`}>
                {services.find(s => s.name === "postgres-cluster-rw")?.running ? "● Running" : "● Stopped"}
              </span>
              {!services.find(s => s.name === "postgres-cluster-rw")?.running ? (
                <button 
                  onClick={startPostgresClusterPortForward}
                  disabled={loading === "postgres-cluster-rw"}
                  className="start-button"
                >
                  {loading === "postgres-cluster-rw" ? "Starting..." : "Start"}
                </button>
              ) : (
                <button 
                  onClick={() => stopPortForward("postgres-cluster-rw")}
                  disabled={loading === "postgres-cluster-rw"}
                  className="stop-button"
                >
                  {loading === "postgres-cluster-rw" ? "Stopping..." : "Stop"}
                </button>
              )}
            </div>
          </div>
        </div>

        <div className="service-group">
          <div className="service-header">
            <div className="service-info">
              <h3>VMKS Grafana (vmks-grafana)</h3>
              <p>Context: hs-gcp-cluster-1 | Namespace: infra | Port: 2998:80</p>
            </div>
            <div className="service-status-controls">
              <span className={`status ${services.find(s => s.name === "vmks-grafana")?.running ? "running" : "stopped"}`}>
                {services.find(s => s.name === "vmks-grafana")?.running ? "● Running" : "● Stopped"}
              </span>
              {!services.find(s => s.name === "vmks-grafana")?.running ? (
                <button 
                  onClick={startVmksGrafanaPortForward}
                  disabled={loading === "vmks-grafana"}
                  className="start-button"
                >
                  {loading === "vmks-grafana" ? "Starting..." : "Start"}
                </button>
              ) : (
                <button 
                  onClick={() => stopPortForward("vmks-grafana")}
                  disabled={loading === "vmks-grafana"}
                  className="stop-button"
                >
                  {loading === "vmks-grafana" ? "Stopping..." : "Stop"}
                </button>
              )}
            </div>
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
