export function getErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message
  }

  if (typeof error === 'string') {
    return error
  }

  if (typeof error === 'object' && error !== null) {
    const maybeMessage = (error as Record<string, unknown>).message
    if (typeof maybeMessage === 'string' && maybeMessage.trim().length > 0) {
      return maybeMessage
    }

    const maybeError = (error as Record<string, unknown>).error
    if (typeof maybeError === 'string' && maybeError.trim().length > 0) {
      return maybeError
    }
  }

  return String(error)
}

export function normalizeInvokeError(error: unknown, fallback: string): Error {
  const message = getErrorMessage(error)
  return new Error(message || fallback)
}
