type FormActionsProps = {
  isEditing: boolean
  onCancel: () => void
}

export let FormActions = ({ isEditing, onCancel }: FormActionsProps) => (
  <div className="form-actions">
    <button type="button" onClick={onCancel} className="cancel-button">
      Cancel
    </button>
    <button type="submit" className="primary-button">
      {isEditing ? "Save" : "Add"}
    </button>
  </div>
)
