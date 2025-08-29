import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import ServiceCard from "./ServiceCard";
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
      const errorMessage = `${error}`;
      setMessage(`Error: ${errorMessage}`);
      
      // If error indicates port forwarding is not running, reset the service state to stopped
      if (errorMessage.includes("port forwarding is not running")) {
        setServices(prev => prev.map(service => 
          service.name === serviceName ? { ...service, running: false } : service
        ));
      }
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
        
        <ServiceCard
          name="db-s"
          displayName="DB Service (db-s)"
          context="hs-docn-cluster-1"
          namespace="monitoring"
          ports="Ports: 5332:25060, 5333:25061, 5334:25062, 5335:25063, 5336:25064"
          isRunning={services.find(s => s.name === "db-s")?.running || false}
          isLoading={loading === "db-s"}
          onStart={startDbPortForward}
          onStop={() => stopPortForward("db-s")}
        />

        <ServiceCard
          name="grafana-docn"
          displayName="Grafana (grafana-docn)"
          context="hs-docn-cluster-1"
          namespace="monitoring"
          ports="Port: 2999:80"
          isRunning={services.find(s => s.name === "grafana-docn")?.running || false}
          isLoading={loading === "grafana-docn"}
          onStart={startGrafanaPortForward}
          onStop={() => stopPortForward("grafana-docn")}
        />

        <ServiceCard
          name="postgres-cluster-rw"
          displayName="Postgres Cluster (postgres-cluster-rw)"
          context="tgs"
          namespace="infra"
          ports="Port: 8100:5432"
          isRunning={services.find(s => s.name === "postgres-cluster-rw")?.running || false}
          isLoading={loading === "postgres-cluster-rw"}
          onStart={startPostgresClusterPortForward}
          onStop={() => stopPortForward("postgres-cluster-rw")}
        />

        <ServiceCard
          name="vmks-grafana"
          displayName="VMKS Grafana (vmks-grafana)"
          context="gke_boxwood-theory-461104-s3_us-east4_cluster-1"
          namespace="infra"
          ports="Port: 2998:80"
          isRunning={services.find(s => s.name === "vmks-grafana")?.running || false}
          isLoading={loading === "vmks-grafana"}
          onStart={startVmksGrafanaPortForward}
          onStop={() => stopPortForward("vmks-grafana")}
        />
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
