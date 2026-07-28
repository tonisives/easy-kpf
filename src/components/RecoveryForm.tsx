import { useState } from "react"
import { RecoverySettings } from "../hooks/hooks"

type RecoveryFormProps = {
  recovery?: RecoverySettings
}

export let RecoveryForm = ({ recovery }: RecoveryFormProps) => {
  let [customized, setCustomized] = useState(Boolean(recovery))
  let hook = recovery?.hooks.before_reconnect[0]
  let [hookType, setHookType] = useState<"command" | "ssh">(hook?.type || "command")

  return (
    <section className="embedded-settings-section">
      <div className="setup-section-heading">
        <div>
          <h3>Recovery</h3>
          <p>Override automatic recovery for this port forward.</p>
        </div>
        <label className="checkbox-label">
          <input
            type="checkbox"
            name="recoveryOverride"
            checked={customized}
            onChange={(event) => setCustomized(event.target.checked)}
          />
          Custom
        </label>
      </div>

      {customized && (
        <>
          <label className="checkbox-label recovery-enabled">
            <input
              type="checkbox"
              name="recoveryEnabled"
              defaultChecked={recovery?.reconnect.enabled ?? true}
            />
            Automatically reconnect this forward
          </label>

          <div className="settings-number-grid">
            <div className="form-group">
              <label>Initial Delay (ms)</label>
              <input
                type="number"
                name="recoveryInitialDelayMs"
                min="0"
                step="1"
                defaultValue={recovery?.reconnect.initial_delay_ms ?? 1000}
              />
            </div>
            <div className="form-group">
              <label>Maximum Delay (ms)</label>
              <input
                type="number"
                name="recoveryMaxDelayMs"
                min="0"
                step="1"
                defaultValue={recovery?.reconnect.max_delay_ms ?? 30000}
              />
            </div>
            <div className="form-group">
              <label>Stable After (seconds)</label>
              <input
                type="number"
                name="recoveryStableAfterSeconds"
                min="0"
                step="1"
                defaultValue={recovery?.reconnect.stable_after_seconds ?? 30}
              />
            </div>
            <div className="form-group">
              <label>Maximum Attempts</label>
              <input
                type="number"
                name="recoveryMaxAttempts"
                min="0"
                step="1"
                defaultValue={recovery?.reconnect.max_attempts ?? 0}
              />
              <p className="field-help">Use 0 to retry indefinitely.</p>
            </div>
          </div>

          <div className="form-group">
            <label>Recovery Hook</label>
            <select
              name="recoveryHookType"
              value={hookType}
              onChange={(event) => setHookType(event.target.value as "command" | "ssh")}
            >
              <option value="command">Local command or script</option>
              <option value="ssh">SSH command</option>
            </select>
          </div>

          {hookType === "ssh" && (
            <div className="form-group">
              <label>SSH Host</label>
              <input
                type="text"
                name="recoveryHookSshHost"
                defaultValue={hook?.ssh_host || ""}
                placeholder="user@hostname or SSH config host"
              />
              <p className="field-help">Uses BatchMode and the existing SSH configuration.</p>
            </div>
          )}

          <div className="form-group">
            <label>{hookType === "ssh" ? "Remote Command" : "Command or Script"}</label>
            <input
              type="text"
              name="recoveryHookCommand"
              defaultValue={hook?.command || ""}
              placeholder={hookType === "ssh" ? "sudo systemctl restart service" : "/path/to/recover"}
            />
            <p className="field-help">Runs before EasyKpf starts the replacement forward.</p>
          </div>

          <div className="form-group">
            <label>Arguments</label>
            <textarea
              name="recoveryHookArgs"
              defaultValue={hook?.args.join("\n") || ""}
              placeholder="One argument per line"
            />
          </div>

          <div className="settings-number-grid">
            <div className="form-group">
              <label>Hook Timeout (seconds)</label>
              <input
                type="number"
                name="recoveryHookTimeoutSeconds"
                min="1"
                step="1"
                defaultValue={hook?.timeout_seconds ?? 30}
              />
            </div>
            <div className="form-group">
              <label>Hook Cooldown (seconds)</label>
              <input
                type="number"
                name="recoveryHookCooldownSeconds"
                min="0"
                step="1"
                defaultValue={hook?.cooldown_seconds ?? 30}
              />
            </div>
          </div>
        </>
      )}
    </section>
  )
}
