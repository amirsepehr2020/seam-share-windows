import QRCode from 'qrcode'

export async function pairingQrDataUrl(payload: unknown): Promise<string> {
  return QRCode.toDataURL(JSON.stringify(payload), { width: 280, margin: 2, errorCorrectionLevel: 'M' })
}
