import { useState, useRef } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { CheckCircle2, ChevronRight, FileText, FolderOpen, Image, Link2, MoreHorizontal, Monitor, Plus, RefreshCw, Send, Smartphone, UploadCloud, Wifi } from 'lucide-react'

type Device = { name: string; ip: string; port: number }
const transfers = [{ name: 'Project-brief.pdf', size: '2.4 MB', time: 'Just now', type: 'pdf' }, { name: 'IMG_2048.jpg', size: '4.8 MB', time: '8 min ago', type: 'image' }, { name: 'notes.txt', size: '18 KB', time: 'Yesterday', type: 'text' }]

export default function App() {
  const input = useRef<HTMLInputElement>(null)
  const [dragging, setDragging] = useState(false)
  const [selected, setSelected] = useState<string | null>(null)
  const [devices, setDevices] = useState<Device[]>([])
  const [scanning, setScanning] = useState(false)
  const [localIp, setLocalIp] = useState('Local network')

  const pick = () => input.current?.click()
  const files = (list: FileList | null) => { const file = list?.[0]; if (file) setSelected(file.name) }
  const scan = async () => {
    setScanning(true)
    try {
      const info = await invoke<{ local_ip: string }>('network_info')
      setLocalIp(info.local_ip)
      const found = await invoke<Device[]>('discover_devices', { deviceName: "Sepehr's PC", timeoutMs: 900 })
      setDevices(found)
    } catch { setDevices([]) }
    finally { setScanning(false) }
  }

  return <main className="shell">
    <div className="glow glow-a"/><div className="glow glow-b"/>
    <header className="topbar"><div className="brand"><img src="/assets/seam-share-logo.svg"/><div><strong>SEAM Share</strong><span>Move freely. Stay connected.</span></div></div><div className="status"><i/><span>Ready to share</span><button aria-label="More"><MoreHorizontal size={18}/></button></div></header>
    <section className="hero"><div className="hero-copy"><h1>Share without<br/><em>the friction.</em></h1><p>Send files, photos and text between your devices. Fast, private, effortless.</p></div><div className="space-card"><div className="space-orbit"><div className="orbit-ring ring-1"/><div className="orbit-ring ring-2"/><div className="device-pill phone"><Smartphone size={20}/><span>Android</span></div><div className="device-pill pc"><Monitor size={20}/><span>This PC</span></div><div className="center-logo"><Wifi size={27}/></div></div></div></section>
    <section className="workspace"><div className={`dropzone ${dragging ? 'dragging' : ''}`} onClick={pick} onDragOver={e => { e.preventDefault(); setDragging(true) }} onDragLeave={() => setDragging(false)} onDrop={e => { e.preventDefault(); setDragging(false); files(e.dataTransfer.files) }}><input ref={input} hidden type="file" multiple onChange={e => files(e.target.files)}/><div className="drop-icon"><UploadCloud size={28}/></div><h2>{selected ?? 'Drop anything here'}</h2><p>{selected ? 'Ready to send to your Android device' : 'Drag & drop files, or click to browse'}</p><div className="drop-actions"><button className="primary" onClick={e => { e.stopPropagation(); pick() }}><Plus size={17}/> Choose files</button><span><Link2 size={14}/> Private connection</span></div></div>
      <aside className="side"><div className="side-head"><div><span className="eyebrow">NEARBY</span><h3>Your devices</h3></div><button aria-label="Scan" onClick={scan}><RefreshCw size={16} className={scanning ? 'spin' : ''}/></button></div>
        {devices.length ? devices.map(device => <div className="device active" key={device.ip}><div className="device-icon"><Smartphone size={20}/></div><div className="device-info"><strong>{device.name}</strong><span><i/> {device.ip}</span></div><ChevronRight size={17}/></div>) : <div className="empty-device"><Smartphone size={20}/><span>{scanning ? 'Scanning your network…' : 'No devices found yet'}<small>Tap ↻ to scan nearby devices</small></span></div>}
        <div className="privacy"><div><Wifi size={18}/></div><span><strong>Local & private</strong><small>{localIp} · transfers stay on your network.</small></span></div></aside></section>
    <section className="recent"><div className="recent-head"><div><span className="eyebrow">ACTIVITY</span><h3>Recent transfers</h3></div><button>View all <ChevronRight size={15}/></button></div><div className="transfer-list">{transfers.map(t => <div className="transfer" key={t.name}><div className="file-icon">{t.type === 'image' ? <Image size={18}/> : t.type === 'text' ? <FileText size={18}/> : <FolderOpen size={18}/>}</div><div className="file-meta"><strong>{t.name}</strong><span>{t.size} · {t.time}</span></div><div className="transfer-state"><CheckCircle2 size={17}/><span>Sent</span></div><button aria-label="Send again"><Send size={16}/></button></div>)}</div></section>
    <footer><span>SEAM Share 0.1</span><span>Connected via local network · {localIp}</span></footer>
  </main>
}
