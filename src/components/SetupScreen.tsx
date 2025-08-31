import { useState, useEffect } from "react"
import { invoke } from "@tauri-apps/api/core"

type SetupScreenProps = {
  onSetupComplete: () => void
}

let SetupScreen = ({ onSetupComplete }: SetupScreenProps) => {
  let [kubectlPath, setKubectlPath] = useState("")
  let [isDetecting, setIsDetecting] = useState(false)
  let [isValidating, setIsValidating] = useState(false)
  let [error, setError] = useState("")
  let [isValid, setIsValid] = useState(false)
  let [kubeconfigPath, setKubeconfigPath] = useState<string | null>(null)
  let [editableKubeconfigPath, setEditableKubeconfigPath] = useState("")
  let [isSettingKubeconfig, setIsSettingKubeconfig] = useState(false)
  let [kubeconfigError, setKubeconfigError] = useState("")
  let [availableContexts, setAvailableContexts] = useState<string[]>([])
  let [isLoadingContexts, setIsLoadingContexts] = useState(false)

  let detectKubectl = async () => {
    setIsDetecting(true)
    setError("")

    try {
      let detectedPath = await invoke<string>("detect_kubectl_path")
      setKubectlPath(detectedPath)
      await validatePath(detectedPath)
    } catch (err) {
      setError(`kubectl not found automatically: ${err}`)
    } finally {
      setIsDetecting(false)
    }
  }

  let validatePath = async (path: string) => {
    if (!path.trim()) {
      setIsValid(false)
      return
    }

    setIsValidating(true)
    setError("")

    try {
      let valid = await invoke<boolean>("validate_kubectl_path", { path })
      setIsValid(valid)
      if (!valid) {
        setError("Invalid kubectl path or kubectl not working")
      }
    } catch (err) {
      setError(`Validation error: ${err}`)
      setIsValid(false)
    } finally {
      setIsValidating(false)
    }
  }

  let handlePathChange = (newPath: string) => {
    setKubectlPath(newPath)
    setIsValid(false)
    if (newPath.trim()) {
      validatePath(newPath)
    }
  }

  let handleSave = async () => {
    if (!isValid || !kubectlPath.trim()) return

    try {
      await invoke("set_kubectl_path", { path: kubectlPath })
      onSetupComplete()
    } catch (err) {
      setError(`Failed to save kubectl path: ${err}`)
    }
  }

  let detectKubeconfig = async () => {
    try {
      let path = await invoke<string | null>("get_kubeconfig_env")
      setKubeconfigPath(path)
      setEditableKubeconfigPath(path || "")
    } catch (err) {
      console.error("Failed to detect KUBECONFIG:", err)
      setKubeconfigPath(null)
      setEditableKubeconfigPath("")
    }
  }

  let handleKubeconfigChange = (newPath: string) => {
    setEditableKubeconfigPath(newPath)
    setKubeconfigError("")
  }

  let handleSetKubeconfig = async () => {
    if (!editableKubeconfigPath.trim()) return

    setIsSettingKubeconfig(true)
    setKubeconfigError("")

    try {
      await invoke("set_kubeconfig_env", { path: editableKubeconfigPath })
      setKubeconfigPath(editableKubeconfigPath)
      loadContexts() // Reload contexts after setting new KUBECONFIG
    } catch (err) {
      setKubeconfigError(`Failed to set KUBECONFIG: ${err}`)
    } finally {
      setIsSettingKubeconfig(false)
    }
  }

  let loadContexts = async () => {
    setIsLoadingContexts(true)
    try {
      let contexts = await invoke<string[]>("get_kubectl_contexts")
      setAvailableContexts(contexts)
    } catch (err) {
      console.error("Failed to load contexts:", err)
      setAvailableContexts([])
    } finally {
      setIsLoadingContexts(false)
    }
  }

  useEffect(() => {
    detectKubectl()
    detectKubeconfig()
    loadContexts()
  }, [])

  return (
    <div style={{ padding: "20px", maxWidth: "600px", margin: "0 auto" }}>
      <h2>Setup kubectl</h2>
      <p>We need to locate kubectl on your system to manage port forwarding.</p>
      {!isValid || isValidating ? (
        <div style={{ marginBottom: "20px" }}>
          <button
            onClick={detectKubectl}
            disabled={isDetecting}
            style={{
              padding: "10px 20px",
              marginBottom: "10px",
              backgroundColor: "#007acc",
              color: "white",
              border: "none",
              borderRadius: "4px",
              cursor: isDetecting ? "not-allowed" : "pointer",
            }}
          >
            {isDetecting ? "Detecting..." : "Auto-detect kubectl"}
          </button>
        </div>
      ) : null}

      <div style={{ marginBottom: "20px" }}>
        <label style={{ display: "block", marginBottom: "5px" }}>kubectl path:</label>
        <input
          type="text"
          value={kubectlPath}
          onChange={(e) => handlePathChange(e.target.value)}
          placeholder="/opt/homebrew/bin/kubectl"
          style={{
            width: "100%",
            padding: "10px",
            border: `2px solid ${isValid ? "#4caf50" : error ? "#f44336" : "#ddd"}`,
            borderRadius: "4px",
            fontSize: "14px",
          }}
        />
        {isValidating && <p style={{ color: "#666", margin: "5px 0" }}>Validating...</p>}
        {isValid && (
          <p style={{ color: "#4caf50", margin: "5px 0" }}>✓ kubectl found and working</p>
        )}
        {error && <p style={{ color: "#f44336", margin: "5px 0" }}>{error}</p>}
      </div>

      {!isValid || isValidating ? (
        <div style={{ marginTop: "20px", fontSize: "12px", color: "#666" }}>
          <p>
            <strong>Common kubectl locations:</strong>
          </p>
          <ul>
            <li>/opt/homebrew/bin/kubectl (Homebrew on Apple Silicon)</li>
            <li>/usr/local/bin/kubectl (Homebrew on Intel Mac)</li>
            <li>/usr/bin/kubectl (System installation)</li>
            <li>/snap/bin/kubectl (Snap on Linux)</li>
          </ul>
        </div>
      ) : null}

      {/* Separator */}
      <hr
        style={{
          margin: "30px 0",
          border: "none",
          borderTop: "1px solid #ddd",
        }}
      />

      {/* KUBECONFIG Section */}
      <h3 style={{ marginBottom: "10px", fontSize: "18px" }}>KUBECONFIG Configuration</h3>
      <p style={{ marginBottom: "20px", color: "#666" }}>
        Configure which Kubernetes config file to use.
      </p>

      <div style={{ marginBottom: "20px" }}>
        <label style={{ display: "block", marginBottom: "5px" }}>KUBECONFIG path:</label>
        <input
          type="text"
          value={editableKubeconfigPath}
          onChange={(e) => handleKubeconfigChange(e.target.value)}
          placeholder={kubeconfigPath || "Not set (using default ~/.kube/config)"}
          style={{
            width: "100%",
            padding: "10px",
            border: `2px solid ${kubeconfigPath ? "#4caf50" : kubeconfigError ? "#f44336" : "#ddd"}`,
            borderRadius: "4px",
            fontSize: "14px",
          }}
        />
        {isSettingKubeconfig && (
          <p style={{ color: "#666", margin: "5px 0" }}>Setting KUBECONFIG...</p>
        )}
        {kubeconfigPath && !kubeconfigError && (
          <p style={{ color: "#4caf50", margin: "5px 0" }}>✓ KUBECONFIG environment variable set</p>
        )}
        {!kubeconfigPath && !kubeconfigError && (
          <p style={{ color: "#ff9800", margin: "5px 0" }}>
            ⚠️ KUBECONFIG not set, using default config
          </p>
        )}
        {kubeconfigError && <p style={{ color: "#f44336", margin: "5px 0" }}>{kubeconfigError}</p>}
        {editableKubeconfigPath !== (kubeconfigPath || "") && (
          <button
            onClick={handleSetKubeconfig}
            disabled={!editableKubeconfigPath.trim() || isSettingKubeconfig}
            style={{
              padding: "8px 16px",
              backgroundColor: editableKubeconfigPath.trim() ? "#4caf50" : "#ccc",
              color: "white",
              border: "none",
              borderRadius: "4px",
              cursor: editableKubeconfigPath.trim() ? "pointer" : "not-allowed",
              marginTop: "10px",
            }}
          >
            {isSettingKubeconfig ? "Setting..." : "Set KUBECONFIG"}
          </button>
        )}
      </div>

      {/* Available Contexts */}
      {(kubeconfigPath || availableContexts.length > 0) && (
        <div style={{ marginBottom: "20px" }}>
          <label style={{ display: "block", marginBottom: "5px", fontWeight: "bold" }}>
            Available Contexts:
          </label>
          {isLoadingContexts ? (
            <p style={{ color: "#666", margin: "5px 0" }}>Loading contexts...</p>
          ) : availableContexts.length > 0 ? (
            <div
              style={{
                border: "1px solid #666666",
                borderRadius: "4px",
                padding: "10px",
                backgroundColor: "darkgray",
                maxHeight: "120px",
                overflowY: "auto",
              }}
            >
              {availableContexts.map((context, index) => (
                <div
                  key={index}
                  style={{
                    padding: "2px 0",
                    fontSize: "14px",
                    color: "#333",
                  }}
                >
                  • {context}
                </div>
              ))}
            </div>
          ) : (
            <p style={{ color: "#ff9800", margin: "5px 0" }}>No contexts found</p>
          )}
        </div>
      )}

      {/* Separator */}
      <hr
        style={{
          margin: "30px 0",
          border: "none",
          borderTop: "1px solid #ddd",
        }}
      />

      <div style={{ display: "flex", width: "100%", justifyContent: "start" }}>
        <button
          onClick={handleSave}
          disabled={!isValid || isValidating}
          style={{
            padding: "10px 20px",
            backgroundColor: isValid ? "#4caf50" : "#ccc",
            color: "white",
            border: "none",
            borderRadius: "4px",
            cursor: isValid ? "pointer" : "not-allowed",
          }}
        >
          Continue
        </button>
      </div>
    </div>
  )
}

export default SetupScreen

