import { useState, useEffect } from "react"
import { invoke } from "@tauri-apps/api/core"

type SetupScreenProps = {
  onSetupComplete: () => void
  onCancel?: () => void
  isDialog?: boolean
}

let SetupScreen = ({ onSetupComplete, onCancel, isDialog }: SetupScreenProps) => {
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

  let content = (
    <div className="setup-panel">
      <div className="dialog-heading">
        <h2>{isDialog ? "Kubernetes Settings" : "Set Up EasyKpf"}</h2>
        <p>Choose the command and configuration EasyKpf should use.</p>
      </div>

      <section className="setup-section">
        <div className="setup-section-heading">
          <h3>kubectl</h3>
          {(!isValid || isValidating) && (
            <button type="button" onClick={detectKubectl} disabled={isDetecting}>
              {isDetecting ? "Detecting..." : "Detect Automatically"}
            </button>
          )}
        </div>
        <div className="form-group">
          <label>Command Path</label>
          <input
            type="text"
            value={kubectlPath}
            onChange={(event) => handlePathChange(event.target.value)}
            placeholder="/opt/homebrew/bin/kubectl"
            className={error ? "input-error" : isValid ? "input-success" : ""}
          />
          {isValidating && <p className="setup-status">Validating...</p>}
          {isValid && <p className="setup-status success">kubectl is available and working.</p>}
          {error && <p className="setup-status error">{error}</p>}
          {!isValid && !isValidating && (
            <p className="field-help">Common locations include /opt/homebrew/bin/kubectl and /usr/local/bin/kubectl.</p>
          )}
        </div>
      </section>

      <section className="setup-section">
        <div className="setup-section-heading">
          <div>
            <h3>Kubernetes Configuration</h3>
            <p>Leave this blank to use ~/.kube/config.</p>
          </div>
        </div>
        <div className="form-group">
          <label>KUBECONFIG Path</label>
          <div className="form-control-row">
            <input
              type="text"
              value={editableKubeconfigPath}
              onChange={(event) => handleKubeconfigChange(event.target.value)}
              placeholder={kubeconfigPath || "Default configuration"}
              className={kubeconfigError ? "input-error" : ""}
            />
            {editableKubeconfigPath !== (kubeconfigPath || "") && (
              <button
                type="button"
                onClick={handleSetKubeconfig}
                disabled={!editableKubeconfigPath.trim() || isSettingKubeconfig}
              >
                {isSettingKubeconfig ? "Applying..." : "Apply"}
              </button>
            )}
          </div>
          {kubeconfigPath && !kubeconfigError && <p className="setup-status success">Custom configuration is active.</p>}
          {!kubeconfigPath && !kubeconfigError && <p className="setup-status">Using the default Kubernetes configuration.</p>}
          {kubeconfigError && <p className="setup-status error">{kubeconfigError}</p>}
        </div>

        {(kubeconfigPath || availableContexts.length > 0) && (
          <div className="context-list-section">
            <label>Available Contexts</label>
            {isLoadingContexts ? (
              <p className="setup-status">Loading contexts...</p>
            ) : availableContexts.length > 0 ? (
              <div className="context-list">
                {availableContexts.map((context) => <div key={context}>{context}</div>)}
              </div>
            ) : (
              <p className="setup-status">No contexts found.</p>
            )}
          </div>
        )}
      </section>

      <div className="dialog-actions">
        {isDialog && onCancel && <button type="button" onClick={onCancel}>Cancel</button>}
        <button type="button" className="primary-button" onClick={handleSave} disabled={!isValid || isValidating}>
          {isDialog ? "Done" : "Continue"}
        </button>
      </div>
    </div>
  )

  if (isDialog) {
    return (
      <div className="settings-modal">{content}</div>
    )
  }

  return <main className="setup-screen">{content}</main>
}

export default SetupScreen
