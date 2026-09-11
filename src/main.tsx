import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import App from './App'
import PairingGate from './PairingGate'
import './styles.css'

createRoot(document.getElementById('root')!).render(<StrictMode><App /><PairingGate /></StrictMode>)
