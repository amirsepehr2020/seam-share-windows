import QRCode from 'qrcode'

type PairingPayload = {
  version: 1
  deviceId: string
  deviceName: string
  address: string
  port: number
  token: string
}

export async function pairingQrDataUrl(payload: unknown): Promise<string> {
  const source = payload as Record<string, unknown>
  const canonical: PairingPayload = {
    version: 1,
    deviceId: String(source.device_id ?? source.deviceId ?? ''),
    deviceName: String(source.device_name ?? source.deviceName ?? ''),
    address: String(source.ip ?? source.address ?? ''),
    port: Number(source.port ?? 0),
    token: String(source.token ?? ''),
  }
  if (!canonical.deviceId || !canonical.deviceName || !canonical.address || !canonical.port || !canonical.token) {
    throw new Error('Invalid pairing information')
  }
  return QRCode.toDataURL(JSON.stringify(canonical), { width: 280, margin: 2, errorCorrectionLevel: 'M' })
}
