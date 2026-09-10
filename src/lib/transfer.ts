export type TransferState = 'queued' | 'transferring' | 'complete' | 'failed'

export type TransferItem = {
  id: string
  name: string
  size: number
  state: TransferState
  progress: number
}

export function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 ** 2) return `${(bytes / 1024).toFixed(1)} KB`
  if (bytes < 1024 ** 3) return `${(bytes / 1024 ** 2).toFixed(1)} MB`
  return `${(bytes / 1024 ** 3).toFixed(1)} GB`
}

export function createTransfer(file: File): TransferItem {
  return { id: crypto.randomUUID(), name: file.name, size: file.size, state: 'queued', progress: 0 }
}
