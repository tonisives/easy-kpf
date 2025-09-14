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
      <label>Name (Optional):</label>
      <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
        <input
          ref={nameInputRef}
          type="text"
          name="name"
          value={nameValue}
          onChange={(e) => onNameChange(e.target.value)}
          placeholder={previewName ? `Will be: ${previewName}` : "Auto-generated from service/host and port"}
          style={{
            flex: 1
          }}
        />
        {(isNameManuallyChanged && !isEditing && previewName) && (
          <button
            type="button"
            onClick={handleReset}
            style={{
              background: "none",
              border: "1px solid #ccc",
              borderRadius: "4px",
              cursor: "pointer",
              padding: "4px 8px",
              fontSize: "12px",
              color: "#666",
              minWidth: "24px",
              height: "24px",
              display: "flex",
              alignItems: "center",
              justifyContent: "center"
            }}
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