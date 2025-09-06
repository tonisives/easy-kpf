import { useState, useCallback } from "react"

type AsyncLoader<T> = {
  data: T[]
  loading: boolean
  load: (...args: any[]) => Promise<void>
}

export let useAsyncLoader = <T>(
  asyncFn: (...args: any[]) => Promise<T[]>,
  setError?: (error: string) => void
): AsyncLoader<T> => {
  let [data, setData] = useState<T[]>([])
  let [loading, setLoading] = useState(false)

  let load = useCallback(async (...args: any[]) => {
    setLoading(true)
    try {
      let result = await asyncFn(...args)
      setData(result)
      setError?.("")
    } catch (error) {
      console.error(`Failed to load data:`, error)
      setError?.(`${error}`)
      setData([])
    } finally {
      setLoading(false)
    }
  }, [asyncFn, setError])

  return { data, loading, load }
}