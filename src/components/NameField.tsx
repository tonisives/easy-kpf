import { useRef } from "react"

type NameFieldProps = {
  nameValue: string
  isNameManuallyChanged: boolean
  previewName: string
  isEditing: boolean
  onNameChange: (name: string) => void
  onReset: () => void
}

export let NameField = ({
  nameValue,
  isNameManuallyChanged,
  previewName,
  isEditing,
  onNameChange,
  onReset,
}: NameFieldProps) => {
  let nameInputRef = useRef<HTMLInputElement>(null)

  let handleReset = () => {
    onReset()
    if (nameInputRef.current) {
      nameInputRef.current.value = ""
    }
  }

  return (
    <div className="form-group">
      <label>Name <span className="optional-label">Optional</span></label>
      <div className="form-control-row">
        <input
          ref={nameInputRef}
          type="text"
          name="name"
          value={nameValue}
          onChange={(e) => onNameChange(e.target.value)}
          placeholder={previewName ? `Will be: ${previewName}` : "Auto-generated from service/host and port"}
        />
        {(isNameManuallyChanged && !isEditing && previewName) && (
          <button
            type="button"
            onClick={handleReset}
            className="reset-button"
            title="Reset to auto-generated name"
          >
            ×
          </button>
        )}
      </div>
      <small>Leave empty to auto-generate. Updates as you select service/host and ports.</small>
    </div>
  )
}
