type FormActionsProps = {
  isEditing: boolean
  onCancel: () => void
}

export let FormActions = ({ isEditing, onCancel }: FormActionsProps) => (
  <div className="form-actions">
    <button type="submit">
      {isEditing ? "Update Configuration" : "Add Configuration"}
    </button>
    <button type="button" onClick={onCancel}>
      Cancel
    </button>
  </div>
)