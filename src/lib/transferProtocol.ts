export type TransferProgress = { id: string; name: string; sent: number; total: number; progress: number; state: 'sending' | 'complete' | 'failed' }
export type TransferTarget = { ip: string; port: number; token: string }
export async function sendFile(file: File, target: TransferTarget, onProgress?: (progress: TransferProgress) => void, signal?: AbortSignal) {
  const id = crypto.randomUUID(); const total = file.size
  onProgress?.({ id, name: file.name, sent: 0, total, progress: 0, state: 'sending' })
  try {
    if (signal?.aborted) throw new DOMException('Transfer cancelled', 'AbortError')
    const response = await fetch(`http://${target.ip}:${target.port}/receive`, { method: 'POST', headers: { 'Content-Type': 'application/octet-stream', 'Content-Length': String(total), 'X-Seam-Token': target.token, 'X-File-Name': encodeURIComponent(file.name) }, body: await file.arrayBuffer(), signal })
    if (!response.ok) throw new Error(`Transfer failed (${response.status})`)
    onProgress?.({ id, name: file.name, sent: total, total, progress: 100, state: 'complete' }); return id
  } catch (error) { onProgress?.({ id, name: file.name, sent: 0, total, progress: 0, state: 'failed' }); throw error }
}
