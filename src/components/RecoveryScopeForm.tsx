import { FormEvent, useState } from "react"
import {
  RecoveryScopeKind,
  RecoverySettings,
} from "../hooks/hooks"
import { parseRecovery } from "../hooks/useFormState"
import { ErrorBanner } from "./ErrorBanner"
import { RecoveryForm } from "./RecoveryForm"

type RecoveryScopeFormProps = {
  kind: RecoveryScopeKind
  scopeKey: string
  label: string
  recovery?: RecoverySettings
  error?: string
  onClearError: () => void
  onSave: (
    kind: RecoveryScopeKind,
    key: string,
    recovery?: RecoverySettings,
  ) => Promise<void>
  onClose: () => void
}

export let RecoveryScopeForm = ({
  kind,
  scopeKey,
  label,
  recovery,
  error,
  onClearError,
  onSave,
  onClose,
}: RecoveryScopeFormProps) => {
  let [saving, setSaving] = useState(false)

  let handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault()
    setSaving(true)
    try {
      let settings = parseRecovery(new FormData(event.currentTarget), recovery)
      await onSave(kind, scopeKey, settings)
    } catch {
      setSaving(false)
      return
    }
    setSaving(false)
    onClose()
  }

  return (
    <div className="settings-modal">
      <div className="add-form">
        <div className="dialog-heading">
          <h3>{label} Recovery</h3>
          <p>Shared by forwards in this {kind === "ssh_host" ? "SSH host" : "Kubernetes context"}.</p>
        </div>

        <ErrorBanner error={error} onClearError={onClearError} />

        <form onSubmit={handleSubmit}>
          <RecoveryForm
            recovery={recovery}
            focusOnOpen
            description={`Configure shared recovery for this ${
              kind === "ssh_host" ? "SSH host" : "Kubernetes context"
            }.`}
          />
          <div className="form-actions">
            <button type="button" onClick={onClose}>Cancel</button>
            <button type="submit" className="primary-button" disabled={saving}>
              {saving ? "Saving..." : "Save"}
            </button>
          </div>
        </form>
      </div>
    </div>
  )
}
