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

/**
 * Convert hex string to bytes
 */
export function hexToBytes(hex: string): Uint8Array {
  const bytes = new Uint8Array(hex.length / 2)
  for (let i = 0; i < hex.length; i += 2) {
    bytes[i / 2] = parseInt(hex.substr(i, 2), 16)
  }
  return bytes
}

/**
 * Check if a string is a valid hex string
 */
export function isHexString(str: string): boolean {
  return /^[0-9a-fA-F]+$/.test(str) && str.length % 2 === 0
}

/**
 * Detect content type from filename extension
 */
export function detectContentTypeFromFilename(filename: string): string {
  const ext = filename.split('.').pop()?.toLowerCase()
  const mimeTypes: Record<string, string> = {
    // Images
    'jpg': 'image/jpeg',
    'jpeg': 'image/jpeg',
    'png': 'image/png',
    'gif': 'image/gif',
    'webp': 'image/webp',
    'svg': 'image/svg+xml',
    'bmp': 'image/bmp',
    'ico': 'image/x-icon',
    // Documents
    'pdf': 'application/pdf',
    'doc': 'application/msword',
    'docx': 'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
    'xls': 'application/vnd.ms-excel',
    'xlsx': 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
    'ppt': 'application/vnd.ms-powerpoint',
    'pptx': 'application/vnd.openxmlformats-officedocument.presentationml.presentation',
    // Text
    'txt': 'text/plain',
    'md': 'text/markdown',
    'html': 'text/html',
    'htm': 'text/html',
    'css': 'text/css',
    'csv': 'text/csv',
    // Code
    'js': 'text/javascript',
    'ts': 'text/typescript',
    'json': 'application/json',
    'xml': 'application/xml',
    'yaml': 'application/x-yaml',
    'yml': 'application/x-yaml',
    // Audio/Video
    'mp3': 'audio/mpeg',
    'wav': 'audio/wav',
    'ogg': 'audio/ogg',
    'mp4': 'video/mp4',
    'webm': 'video/webm',
    'avi': 'video/x-msvideo',
    // Archives
    'zip': 'application/zip',
    'tar': 'application/x-tar',
    'gz': 'application/gzip',
    '7z': 'application/x-7z-compressed',
    'rar': 'application/vnd.rar',
  }
  return mimeTypes[ext || ''] || 'application/octet-stream'
}

/**
 * Parse a private key from file bytes
 * Supports: 32-byte raw binary, 64-char hex string
 */
export async function parsePrivateKey(file: File): Promise<Uint8Array> {
  const bytes = new Uint8Array(await file.arrayBuffer())

  // Raw 32 bytes
  if (bytes.length === 32) {
    return bytes
  }

  // Try hex-encoded (with or without whitespace/newlines)
  const text = new TextDecoder().decode(bytes).trim()
  const cleanHex = text.replace(/\s+/g, '')
  
  if (cleanHex.length === 64 && isHexString(cleanHex)) {
    return hexToBytes(cleanHex)
  }

  throw new Error('Invalid key format: expected 32 raw bytes or 64 hex characters')
}

/**
 * Decode base64 string to bytes
 */
export function base64ToBytes(base64: string): Uint8Array {
  const binaryString = atob(base64)
  const bytes = new Uint8Array(binaryString.length)
  for (let i = 0; i < binaryString.length; i++) {
    bytes[i] = binaryString.charCodeAt(i)
  }
  return bytes
}

/**
 * Check if a string is valid base64
 */
export function isBase64(str: string): boolean {
  try {
    return btoa(atob(str)) === str
  } catch {
    return false
  }
}

/**
 * Parse a certificate from file bytes
 * Supports: raw CBOR binary, base64-encoded CBOR
 */
export function parseCertificateBytes(bytes: Uint8Array): Uint8Array {
  // Check if it looks like text (base64)
  // Base64 strings are ASCII printable characters
  const isText = bytes.every(b => b >= 32 && b < 127)
  
  if (isText) {
    const text = new TextDecoder().decode(bytes).trim()
    
    // Try base64 decoding
    try {
      const decoded = base64ToBytes(text)
      // Verify it's valid CBOR by checking it starts with a map marker (0xa0-0xbf or 0xbf)
      // CBOR maps start with 0xa0-0xbf for small maps or 0xb9/0xba/0xbb for larger
      if (decoded.length > 0 && (decoded[0] >= 0xa0 || decoded[0] === 0xbf)) {
        return decoded
      }
      // If first byte check fails, still return decoded - let WASM parser give proper error
      return decoded
    } catch {
      // Not valid base64
    }
  }
  
  // Return as-is (assume raw CBOR)
  return bytes
}

/**
 * Trigger file download in browser
 */
export function downloadFile(bytes: Uint8Array, filename: string): void {
  const blob = new Blob([bytes], { type: 'application/octet-stream' })
  const url = URL.createObjectURL(blob)
  
  const link = document.createElement('a')
  link.href = url
  link.download = filename
  document.body.appendChild(link)
  link.click()
  document.body.removeChild(link)
  
  // Clean up the blob URL after a short delay
  setTimeout(() => URL.revokeObjectURL(url), 1000)
}
