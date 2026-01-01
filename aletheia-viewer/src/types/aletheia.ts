/**
 * Aletheia file format types
 */

export interface ByteRange {
  start: number
  end: number
}

export interface Header {
  creator_id: string
  signed_at: number
  content_type?: string
  original_name?: string
  description?: string
  custom?: Record<string, unknown>
}

export interface Certificate {
  version: number
  serial: Uint8Array
  subject_id: string
  subject_name: string
  public_key: Uint8Array
  issuer_id: string
  issued_at: number
  is_ca: boolean
  signature: Uint8Array
}

export interface Flags {
  raw: Uint8Array
  isCompressed: boolean
}

export interface AletheiaFile {
  // Raw data
  rawBytes: Uint8Array

  // Parsed fields with byte ranges
  magic: { value: Uint8Array; range: ByteRange }
  version: { major: number; minor: number; range: ByteRange }
  flags: { value: Flags; range: ByteRange }
  header: { value: Header; raw: Uint8Array; range: ByteRange }
  payload: { value: Uint8Array; range: ByteRange }
  certificateChain: { value: Certificate[]; raw: Uint8Array; range: ByteRange }
  signature: { value: Uint8Array; range: ByteRange }

  // Computed
  signatureInput: Uint8Array
}

export interface VerificationStep {
  id: string
  label: string
  status: 'pending' | 'success' | 'error'
  error?: string
}

export interface VerificationResult {
  isValid: boolean
  steps: VerificationStep[]
}

export interface ParseError {
  message: string
  offset?: number
}
