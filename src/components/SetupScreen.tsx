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

  useEffect(() => {
    detectKubectl()
  }, [])

  return (
    <div style={{ padding: "20px", maxWidth: "600px", margin: "0 auto" }}>
      <h2>Setup kubectl</h2>
      <p>We need to locate kubectl on your system to manage port forwarding.</p>
      
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
            cursor: isDetecting ? "not-allowed" : "pointer"
          }}
        >
          {isDetecting ? "Detecting..." : "Auto-detect kubectl"}
        </button>
      </div>

      <div style={{ marginBottom: "20px" }}>
        <label style={{ display: "block", marginBottom: "5px" }}>
          kubectl path:
        </label>
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
            fontSize: "14px"
          }}
        />
        {isValidating && <p style={{ color: "#666", margin: "5px 0" }}>Validating...</p>}
        {isValid && <p style={{ color: "#4caf50", margin: "5px 0" }}>✓ kubectl found and working</p>}
        {error && <p style={{ color: "#f44336", margin: "5px 0" }}>{error}</p>}
      </div>

      <div>
        <button
          onClick={handleSave}
          disabled={!isValid || isValidating}
          style={{
            padding: "10px 20px",
            backgroundColor: isValid ? "#4caf50" : "#ccc",
            color: "white",
            border: "none",
            borderRadius: "4px",
            cursor: isValid ? "pointer" : "not-allowed"
          }}
        >
          Continue
        </button>
      </div>

      <div style={{ marginTop: "20px", fontSize: "12px", color: "#666" }}>
        <p><strong>Common kubectl locations:</strong></p>
        <ul>
          <li>/opt/homebrew/bin/kubectl (Homebrew on Apple Silicon)</li>
          <li>/usr/local/bin/kubectl (Homebrew on Intel Mac)</li>
          <li>/usr/bin/kubectl (System installation)</li>
          <li>/snap/bin/kubectl (Snap on Linux)</li>
        </ul>
      </div>
    </div>
  )
}

export default SetupScreen