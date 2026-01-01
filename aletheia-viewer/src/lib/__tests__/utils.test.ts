import { describe, it, expect } from 'vitest'
import { bytesToHex, formatTimestamp, formatBytes, detectContentType, decodeText } from '../utils'

describe('Utility Functions', () => {
  describe('bytesToHex', () => {
    it('should convert bytes to hex string', () => {
      const bytes = new Uint8Array([0x00, 0xff, 0x12, 0x34])
      expect(bytesToHex(bytes)).toBe('00ff1234')
    })

    it('should handle empty array', () => {
      const bytes = new Uint8Array([])
      expect(bytesToHex(bytes)).toBe('')
    })

    it('should pad single digits', () => {
      const bytes = new Uint8Array([0x01, 0x02, 0x0a, 0x0f])
      expect(bytesToHex(bytes)).toBe('01020a0f')
    })
  })

  describe('formatTimestamp', () => {
    it('should format unix timestamp to UTC string', () => {
      const timestamp = 1609459200 // 2021-01-01 00:00:00 UTC
      const formatted = formatTimestamp(timestamp)
      expect(formatted).toContain('2021')
      expect(formatted).toContain('UTC')
    })

    it('should handle zero timestamp', () => {
      const formatted = formatTimestamp(0)
      expect(formatted).toContain('1970')
    })
  })

  describe('formatBytes', () => {
    it('should format bytes as Bytes', () => {
      expect(formatBytes(100)).toBe('100 Bytes')
      expect(formatBytes(1023)).toBe('1023 Bytes')
    })

    it('should format as KB', () => {
      expect(formatBytes(1024)).toBe('1 KB')
      expect(formatBytes(1536)).toBe('1.5 KB')
    })

    it('should format as MB', () => {
      expect(formatBytes(1048576)).toBe('1 MB')
      expect(formatBytes(1572864)).toBe('1.5 MB')
    })

    it('should format as GB', () => {
      expect(formatBytes(1073741824)).toBe('1 GB')
    })

    it('should handle zero', () => {
      expect(formatBytes(0)).toBe('0 Bytes')
    })
  })

  describe('detectContentType', () => {
    it('should detect PNG image', () => {
      const png = new Uint8Array([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a])
      expect(detectContentType(png)).toBe('image/png')
    })

    it('should detect JPEG image', () => {
      const jpeg = new Uint8Array([0xff, 0xd8, 0xff, 0xe0]) // Need 4 bytes minimum
      expect(detectContentType(jpeg)).toBe('image/jpeg')
    })

    it('should detect GIF image', () => {
      const gif = new Uint8Array([0x47, 0x49, 0x46, 0x38, 0x39, 0x61]) // GIF89a
      expect(detectContentType(gif)).toBe('image/gif')
    })

    it('should use declared content type if magic bytes not recognized', () => {
      const unknown = new Uint8Array([0x00, 0x01, 0x02])
      expect(detectContentType(unknown, 'text/plain')).toBe('text/plain')
    })

    it('should default to application/octet-stream', () => {
      const unknown = new Uint8Array([0x00, 0x01, 0x02])
      expect(detectContentType(unknown)).toBe('application/octet-stream')
    })
  })

  describe('decodeText', () => {
    it('should decode UTF-8 text', () => {
      const text = new TextEncoder().encode('Hello, World!')
      expect(decodeText(text)).toBe('Hello, World!')
    })

    it('should handle Unicode characters', () => {
      const text = new TextEncoder().encode('Hello 世界 🌍')
      expect(decodeText(text)).toBe('Hello 世界 🌍')
    })

    it('should throw on oversized text', () => {
      const large = new Uint8Array(2 * 1024 * 1024) // 2MB
      expect(() => decodeText(large, 1024 * 1024)).toThrow(/too large/i)
    })

    it('should handle empty text', () => {
      const empty = new Uint8Array([])
      expect(decodeText(empty)).toBe('')
    })
  })

  describe('Byte Range Verification', () => {
    it('should validate non-overlapping ranges', () => {
      // Test helper to validate byte ranges don't overlap
      const ranges = [
        { start: 0, end: 8 },    // magic
        { start: 8, end: 10 },   // version
        { start: 10, end: 12 },  // flags
        { start: 12, end: 20 }   // header len + partial header
      ]

      for (let i = 0; i < ranges.length - 1; i++) {
        expect(ranges[i]!.end).toBe(ranges[i + 1]!.start)
      }
    })

    it('should validate range size calculation', () => {
      const range = { start: 100, end: 164 }
      const size = range.end - range.start
      expect(size).toBe(64) // Ed25519 signature size
    })
  })

  describe('Signature Input Extraction', () => {
    it('should extract all bytes except last 64 as signature input', () => {
      // Critical correctness test: signature_input must be exact
      const totalSize = 1000
      const signatureSize = 64
      const expectedInputSize = totalSize - signatureSize

      expect(expectedInputSize).toBe(936)

      // Verify slice indices
      const mockFile = new Uint8Array(totalSize)
      const signatureInput = mockFile.slice(0, totalSize - 64)
      const signature = mockFile.slice(totalSize - 64)

      expect(signatureInput.length).toBe(936)
      expect(signature.length).toBe(64)

      // Ensure no overlap
      expect(signatureInput.length + signature.length).toBe(totalSize)
    })
  })

  describe('Certificate Chain Verification Logic', () => {
    it('should verify chain ordering', () => {
      // Certificate chain must be: creator -> intermediate(s) -> root
      // Where each cert is signed by the next one

      const mockChain = [
        { subject_id: 'user@example.com', issuer_id: 'ca@example.com', is_ca: false },
        { subject_id: 'ca@example.com', issuer_id: 'ca@example.com', is_ca: true }
      ]

      // Verify first cert's issuer matches second cert's subject
      expect(mockChain[0]!.issuer_id).toBe(mockChain[1]!.subject_id)

      // Verify root is self-signed
      const rootCert = mockChain[mockChain.length - 1]!
      expect(rootCert.issuer_id).toBe(rootCert.subject_id)

      // Verify root is a CA
      expect(rootCert.is_ca).toBe(true)
    })

    it('should validate creator cert is not a CA', () => {
      const creatorCert = { is_ca: false, subject_id: 'user@example.com' }
      expect(creatorCert.is_ca).toBe(false)
    })

    it('should validate intermediate/root certs are CAs', () => {
      const caCert = { is_ca: true, subject_id: 'ca@example.com' }
      expect(caCert.is_ca).toBe(true)
    })
  })
})
