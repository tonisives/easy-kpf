let errorIdentity = (error: string) =>
  error
    .trim()
    .replace(/^[EWI]\d{4}\s+\d{2}:\d{2}:\d{2}\.\d+\s+\d+\s+[^\]]+\]\s*/, "")
    .replace(/\s+/g, " ")

export let appendLatestServiceError = (errors: string[] | undefined, error: string) => {
  let latest = error.trim()
  if (!latest) return errors

  let identity = errorIdentity(latest)
  return [...(errors || []).filter((existing) => errorIdentity(existing) !== identity), latest]
}
