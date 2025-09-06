import { PortForwardConfig } from "../hooks/hooks"

type KubernetesSelectProps = {
  label: string
  name: string
  value: string
  onChange: (value: string) => void
  options: string[]
  loading: boolean
  disabled?: boolean
  required?: boolean
  loadingText: string
  placeholderText: string
  editingConfig?: {
    config: PortForwardConfig
    index: number
  } | null
  originalValueKey: keyof PortForwardConfig
}

export let KubernetesSelect = ({
  label,
  name,
  value,
  onChange,
  options,
  loading,
  disabled = false,
  required = false,
  loadingText,
  placeholderText,
  editingConfig,
  originalValueKey,
}: KubernetesSelectProps) => {
  let renderOptions = () => {
    let allOptions = [...options]
    let originalValue = editingConfig?.config[originalValueKey] as string

    // Add original value if it's not in available options (unavailable)
    if (originalValue && !options.includes(originalValue)) {
      allOptions.unshift(originalValue)
    }

    return allOptions.map((option) => {
      let isUnavailable = originalValue && option === originalValue && !options.includes(option)
      return (
        <option
          key={option}
          value={option}
          style={{
            color: isUnavailable ? "#888" : "inherit",
            fontStyle: isUnavailable ? "italic" : "normal",
          }}
        >
          {option}{isUnavailable ? " (unavailable)" : ""}
        </option>
      )
    })
  }

  return (
    <div className="form-group">
      <label>{label}:</label>
      <select
        name={name}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        disabled={disabled || loading}
        required={required}
        style={{ opacity: disabled || loading ? 0.6 : 1 }}
      >
        <option value="">{loading ? loadingText : placeholderText}</option>
        {renderOptions()}
      </select>
      {loading && (
        <div className="loading-bar">
          <div className="loading-progress"></div>
        </div>
      )}
    </div>
  )
}