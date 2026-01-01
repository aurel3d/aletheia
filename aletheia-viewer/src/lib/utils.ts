// @ts-nocheck
/**
 * Utility functions for formatting and display
 */

/**
 * Convert bytes to hex string
 */
export function bytesToHex(bytes: Uint8Array): string {
  return Array.from(bytes)
    .map((b) => b.toString(16).padStart(2, '0'))
    .join('')
}

/**
 * Convert bytes to hex string with spaces
 */
export function bytesToHexSpaced(bytes: Uint8Array): string {
  return Array.from(bytes)
    .map((b) => b.toString(16).padStart(2, '0'))
    .join(' ')
}

/**
 * Format Unix timestamp to human-readable string
 */
export function formatTimestamp(timestamp: number): string {
  const date = new Date(timestamp * 1000)
  return date.toISOString().replace('T', ' ').replace(/\.\d+Z$/, ' UTC')
}

/**
 * Format byte size
 */
export function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 Bytes'
  const k = 1024
  const sizes = ['Bytes', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}

/**
 * Compute SHA-256 fingerprint of public key
 */
export async function computeFingerprint(publicKey: Uint8Array): Promise<string> {
  const hashBuffer = await crypto.subtle.digest('SHA-256', publicKey)
  const hashArray = new Uint8Array(hashBuffer)
  return bytesToHex(hashArray)
}

/**
 * Format fingerprint with colons
 */
export function formatFingerprint(hex: string): string {
  return hex.match(/.{1,2}/g)?.join(':').toUpperCase() || hex
}

/**
 * Copy text to clipboard
 */
export async function copyToClipboard(text: string): Promise<void> {
  try {
    await navigator.clipboard.writeText(text)
  } catch (error) {
    // Fallback for older browsers
    const textarea = document.createElement('textarea')
    textarea.value = text
    textarea.style.position = 'fixed'
    textarea.style.opacity = '0'
    document.body.appendChild(textarea)
    textarea.select()
    document.execCommand('copy')
    document.body.removeChild(textarea)
  }
}

/**
 * Detect content type from payload
 */
export function detectContentType(payload: Uint8Array, declaredType?: string): string {
  if (declaredType) return declaredType

  // Check for common file signatures
  if (payload.length >= 4) {
    const sig = payload.slice(0, 4)

    // PNG
    if (sig[0] === 0x89 && sig[1] === 0x50 && sig[2] === 0x4e && sig[3] === 0x47) {
      return 'image/png'
    }

    // JPEG
    if (sig[0] === 0xff && sig[1] === 0xd8 && sig[2] === 0xff) {
      return 'image/jpeg'
    }

    // GIF
    if (sig[0] === 0x47 && sig[1] === 0x49 && sig[2] === 0x46) {
      return 'image/gif'
    }

    // PDF
    if (sig[0] === 0x25 && sig[1] === 0x50 && sig[2] === 0x44 && sig[3] === 0x46) {
      return 'application/pdf'
    }
  }

  // Check if it's text (valid UTF-8 with printable characters)
  try {
    const text = new TextDecoder('utf-8', { fatal: true }).decode(payload.slice(0, Math.min(1024, payload.length)))
    if (text.split('').every((c) => c.charCodeAt(0) >= 32 || c === '\n' || c === '\r' || c === '\t')) {
      return 'text/plain'
    }
  } catch {
    // Not valid UTF-8
  }

  return 'application/octet-stream'
}

/**
 * Safely decode text with size limit
 */
export function decodeText(payload: Uint8Array, maxSize: number = 1024 * 1024): string {
  if (payload.length > maxSize) {
    throw new Error(`Text too large: ${formatBytes(payload.length)} exceeds ${formatBytes(maxSize)}`)
  }

  try {
    return new TextDecoder('utf-8', { fatal: true }).decode(payload)
  } catch (error) {
    throw new Error('Invalid UTF-8 encoding')
  }
}
