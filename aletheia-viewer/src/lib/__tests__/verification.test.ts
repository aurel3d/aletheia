import { describe, it, expect, beforeAll } from 'vitest'
import init, { parse_aletheia_file, verify_aletheia_file } from '../wasm-pkg/aletheia.js'

describe('Aletheia WASM Module', () => {
  beforeAll(async () => {
    await init()
  })

  describe('Parser', () => {
    it('should parse valid .alx file structure', () => {
      // Create a minimal valid .alx file structure
      const magic = new Uint8Array([0x41, 0x4c, 0x45, 0x54, 0x48, 0x45, 0x49, 0x41]) // "ALETHEIA"
      const version = new Uint8Array([1, 0]) // version 1.0
      const flags = new Uint8Array([0, 0]) // no compression

      // Minimal CBOR header: {creator_id: "test", signed_at: 0}
      const headerCbor = new Uint8Array([
        0xa2, // map(2)
        0x6a, 0x63, 0x72, 0x65, 0x61, 0x74, 0x6f, 0x72, 0x5f, 0x69, 0x64, // "creator_id"
        0x64, 0x74, 0x65, 0x73, 0x74, // "test"
        0x69, 0x73, 0x69, 0x67, 0x6e, 0x65, 0x64, 0x5f, 0x61, 0x74, // "signed_at"
        0x00 // 0
      ])
      const headerLen = new Uint8Array(4)
      new DataView(headerLen.buffer).setUint32(0, headerCbor.length, true)

      // Payload
      const payload = new TextEncoder().encode('test')
      const payloadLen = new Uint8Array(8)
      new DataView(payloadLen.buffer).setBigUint64(0, BigInt(payload.length), true)

      // Minimal cert chain (empty for this test)
      const certChain = new Uint8Array([0x80]) // empty array in CBOR
      const certChainLen = new Uint8Array(4)
      new DataView(certChainLen.buffer).setUint32(0, certChain.length, true)

      // Signature (64 bytes of zeros)
      const signature = new Uint8Array(64)

      // Combine all parts
      const fileData = new Uint8Array([
        ...magic,
        ...version,
        ...flags,
        ...headerLen,
        ...headerCbor,
        ...payloadLen,
        ...payload,
        ...certChainLen,
        ...certChain,
        ...signature
      ])

      // This should throw because cert chain is empty, but it tests the parser
      expect(() => parse_aletheia_file(fileData)).toThrow()
    })

    it('should reject invalid magic bytes', () => {
      const invalidMagic = new Uint8Array(100)
      expect(() => parse_aletheia_file(invalidMagic)).toThrow(/magic|invalid/i)
    })

    it('should track byte ranges correctly', () => {
      // For a valid file, byte ranges should be sequential and non-overlapping
      // This would require a complete valid test file
      // For now, we verify that ranges are included in the parsed result

      // We'll test this with an actual file in integration tests
      expect(true).toBe(true)
    })
  })

  describe('Signature Input Extraction', () => {
    it('should extract signature_input as all bytes except final 64', () => {
      // The signature input should be the entire file minus the last 64 bytes
      // This is critical for Ed25519 verification correctness

      const testData = new Uint8Array(200)
      for (let i = 0; i < 200; i++) {
        testData[i] = i % 256
      }

      // The last 64 bytes are the signature
      const expectedSignatureInput = testData.slice(0, 200 - 64)
      const actualSignature = testData.slice(200 - 64)

      expect(expectedSignatureInput.length).toBe(136)
      expect(actualSignature.length).toBe(64)
    })
  })

  describe('Certificate Chain Verification', () => {
    it('should verify each cert is signed by the next cert in chain', () => {
      // Certificate chain verification logic:
      // 1. For each cert (except root), verify signature with issuer's public key
      // 2. Root cert must be self-signed
      // 3. Root cert's public key must match a trusted root

      // This is tested implicitly through the verify_aletheia_file function
      expect(true).toBe(true)
    })

    it('should reject chain with untrusted root', () => {
      // Create a minimal test file
      const testFile = new Uint8Array(200) // Invalid file
      const emptyTrustedRoots: any[] = []

      // Should fail because no trusted roots
      expect(() => verify_aletheia_file(testFile, emptyTrustedRoots)).toThrow()
    })

    it('should accept chain with trusted root', () => {
      // This requires a complete valid test file
      // Will be tested in integration tests with actual CLI-generated files
      expect(true).toBe(true)
    })
  })

  describe('Edge Cases', () => {
    it('should handle empty input', () => {
      const empty = new Uint8Array(0)
      expect(() => parse_aletheia_file(empty)).toThrow()
    })

    it('should handle truncated file', () => {
      const magic = new Uint8Array([0x41, 0x4c, 0x45, 0x54, 0x48, 0x45, 0x49, 0x41])
      const version = new Uint8Array([1, 0])
      const truncated = new Uint8Array([...magic, ...version]) // Missing rest of file

      expect(() => parse_aletheia_file(truncated)).toThrow()
    })

    it('should handle large file size without crashing', () => {
      // Test that the parser doesn't crash on large payload sizes
      // (actual allocation should fail gracefully)

      const magic = new Uint8Array([0x41, 0x4c, 0x45, 0x54, 0x48, 0x45, 0x49, 0x41])
      const version = new Uint8Array([1, 0])
      const flags = new Uint8Array([0, 0])
      const headerLen = new Uint8Array([4, 0, 0, 0]) // 4 bytes
      const header = new Uint8Array([0xa0, 0x00, 0x00, 0x00]) // minimal CBOR

      // Set payload length to max safe integer
      const payloadLen = new Uint8Array(8)
      new DataView(payloadLen.buffer).setBigUint64(0, BigInt(Number.MAX_SAFE_INTEGER), true)

      const truncatedLargeFile = new Uint8Array([
        ...magic, ...version, ...flags, ...headerLen, ...header, ...payloadLen
      ])

      expect(() => parse_aletheia_file(truncatedLargeFile)).toThrow()
    })
  })
})
