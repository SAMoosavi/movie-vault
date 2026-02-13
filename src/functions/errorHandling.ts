import { error as writeErrorLog } from '@tauri-apps/plugin-log'
import { toast } from 'vue3-toastify'
import { getErrorMessage } from '@/functions/errorMessage'

function serializeErrorDetails(error: unknown): string {
  if (error instanceof Error) {
    const details: string[] = []
    if (error.name) {
      details.push(`name=${error.name}`)
    }
    if (error.stack) {
      details.push(`stack=${error.stack}`)
    }
    return details.join(' | ')
  }

  if (typeof error === 'object' && error !== null) {
    try {
      return JSON.stringify(error)
    } catch {
      return String(error)
    }
  }

  return String(error)
}

export function logFrontendError(context: string, error: unknown): string {
  const message = getErrorMessage(error)
  const details = serializeErrorDetails(error)
  const logMessage = `[${context}] ${message}${details ? ` | details=${details}` : ''}`

  console.error(`[${context}]`, error)
  void writeErrorLog(logMessage).catch(() => {
    console.error(`[${context}] Failed to write tauri log`, logMessage)
  })

  return message
}

export function handleFrontendError(context: string, error: unknown, userMessagePrefix?: string): string {
  const message = logFrontendError(context, error)
  const userMessage = userMessagePrefix ? `${userMessagePrefix}: ${message}` : message
  toast.error(userMessage)
  return message
}
