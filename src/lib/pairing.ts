export type PairingPayload = {
  version: 1
  deviceId: string
  deviceName: string
  address?: string
  port?: number
}

export function createPairingPayload(deviceName: string): PairingPayload {
  return { version: 1, deviceId: crypto.randomUUID(), deviceName }
}

export function encodePairing(payload: PairingPayload): string {
  return JSON.stringify(payload)
}

export function decodePairing(value: string): PairingPayload {
  const parsed = JSON.parse(value) as Partial<PairingPayload>
  if (parsed.version !== 1 || typeof parsed.deviceId !== 'string' || typeof parsed.deviceName !== 'string') {
    throw new Error('Invalid SEAM Share pairing payload')
  }
  return parsed as PairingPayload
}
