type ErrorBannerProps = {
  error?: string
  onClearError: () => void
}

export let ErrorBanner = ({ error, onClearError }: ErrorBannerProps) => {
  if (!error) return null

  return (
    <div className="form-error">
      <span className="form-error-text">{error}</span>
      <button
        type="button"
        onClick={onClearError}
        className="form-error-close"
        title="Clear error"
      >
        ×
      </button>
    </div>
  )
}