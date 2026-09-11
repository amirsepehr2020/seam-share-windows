import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { pairingQrDataUrl } from './lib/qr'

type PairingInfo = { device_id: string; device_name: string; ip: string; port: number; token: string }

export default function PairingGate() {
  const [open, setOpen] = useState(false)
  const [qr, setQr] = useState('')
  const [info, setInfo] = useState<PairingInfo | null>(null)
  const [copied, setCopied] = useState(false)

  useEffect(() => {
    let cancelled = false
    const load = async () => {
      try {
        const paired = await invoke<unknown[]>('paired_devices')
        if (cancelled || paired.length > 0) return
        const next = await invoke<PairingInfo>('pairing_info')
        const image = await pairingQrDataUrl(next)
        if (!cancelled) { setInfo(next); setQr(image); setOpen(true) }
      } catch { /* native backend may still be starting */ }
    }
    void load()
    const timer = window.setInterval(() => { if (!open) void load() }, 2500)
    return () => { cancelled = true; window.clearInterval(timer) }
  }, [open])

  if (!open || !info) return null

  const copy = async () => {
    await navigator.clipboard?.writeText(JSON.stringify(info))
    setCopied(true)
    window.setTimeout(() => setCopied(false), 1600)
  }

  return <div className="pairing-gate" role="dialog" aria-modal="true">
    <div className="pairing-gate-card">
      <div className="pairing-gate-kicker">SEAM SHARE · FIRST CONNECTION</div>
      <h2>Connect your Android</h2>
      <p>Keep both devices on the same Wi‑Fi network, then open SEAM Share on Android and scan this QR code.</p>
      <div className="pairing-gate-qr"><img src={qr} alt="SEAM Share pairing QR code" /></div>
      <strong>{info.device_name}</strong>
      <span>{info.ip}:{info.port}</span>
      <div className="pairing-gate-actions">
        <button className="secondary" onClick={()=>void copy()}>{copied ? 'Copied' : 'Copy pairing code'}</button>
        <button className="primary" onClick={()=>setOpen(false)}>Continue</button>
      </div>
    </div>
  </div>
}
