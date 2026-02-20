import { useSshTesting } from "../hooks/useSshTesting"

type SshFormProps = {
  sshHost: string
  sshPort: string
  defaultLocalInterface: string
  onSshHostChange: (host: string) => void
  onSshPortChange: (port: string) => void
}

export let SshForm = ({
  sshHost,
  sshPort,
  defaultLocalInterface,
  onSshHostChange,
  onSshPortChange,
}: SshFormProps) => {
  let { testStatus, testMessage, testSshConnection } = useSshTesting()

  return (
    <>
      <div className="form-group">
        <label>SSH Host:</label>
        <input
          type="text"
          name="sshHost"
          value={sshHost}
          onChange={(e) => onSshHostChange(e.target.value)}
          placeholder="e.g., user@hostname or hostname"
          required
          className={
            testStatus === "success"
              ? "input-success"
              : testStatus === "error"
                ? "input-error"
                : ""
          }
        />
        <small>SSH connection string (user@host or just host)</small>
      </div>

      <div className="form-group">
        <label>Port:</label>
        <input
          type="text"
          name="sshPort"
          value={sshPort}
          onChange={(e) => onSshPortChange(e.target.value)}
          placeholder="e.g., 8080:80"
          required
          className={
            testStatus === "success"
              ? "input-success"
              : testStatus === "error"
                ? "input-error"
                : ""
          }
        />
        <small>Port mapping in format local:remote</small>
      </div>

      <div className="form-group">
        <label>Local Interface (Optional):</label>
        <input
          type="text"
          name="localInterface"
          defaultValue={defaultLocalInterface}
          placeholder="e.g., 127.0.0.2, 0.0.0.0"
        />
        <small>
          Bind to specific interface (default: 127.0.0.1). Will create if doesn't exist.
        </small>
      </div>

      <div className="form-group">
        <button
          type="button"
          className={`test-button ${testStatus === "testing" ? "testing" : ""}`}
          disabled={testStatus === "testing"}
          onClick={() => testSshConnection(sshHost)}
        >
          {testStatus === "testing" ? "Testing..." : "Test Connection"}
        </button>
        <small>Test SSH connection before adding</small>
      </div>

      {testMessage && (
        <div className={`test-feedback ${testStatus}`}>
          <span className="test-icon">
            {testStatus === "success" ? "✓" : testStatus === "error" ? "✗" : "⏳"}
          </span>
          <span className="test-message">{testMessage}</span>
        </div>
      )}
    </>
  )
}