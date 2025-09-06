type PortSuggestionsProps = {
  ports: string[]
  loading: boolean
}

export let PortSuggestions = ({ ports, loading }: PortSuggestionsProps) => {
  let handlePortClick = (port: string) => {
    let portsInput = document.querySelector('input[name="ports"]') as HTMLInputElement
    if (portsInput) {
      let current = portsInput.value.trim()
      portsInput.value = current ? `${current}, ${port}` : port
    }
  }

  if (loading) {
    return (
      <div className="loading-bar">
        <div className="loading-progress"></div>
      </div>
    )
  }

  if (ports.length === 0) {
    return null
  }

  return (
    <div className="suggested-ports">
      <small>Detected ports (click to use):</small>
      <div className="port-suggestions">
        {ports.map((port) => (
          <button
            key={port}
            type="button"
            className="port-suggestion"
            onClick={() => handlePortClick(port)}
          >
            {port}
          </button>
        ))}
      </div>
    </div>
  )
}