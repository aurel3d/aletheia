// @ts-nocheck
/**
 * Binary parser for Aletheia (.alx) files
 *
 * File format:
 * - Magic bytes (8): "ALETHEIA"
 * - Version (2): major, minor
 * - Flags (2): compression flags
 * - Header length (4): u32 LE
 * - Header: CBOR-encoded
 * - Payload length (8): u64 LE
 * - Payload: raw bytes
 * - Certificate chain length (4): u32 LE
 * - Certificate chain: CBOR-encoded
 * - Signature (64): Ed25519 signature
 */

import { decode as decodeCBOR } from 'cbor-x'
import type { AletheiaFile, Header, Certificate, Flags, ByteRange } from '../types/aletheia'

const MAGIC_BYTES = new Uint8Array([0x41, 0x4c, 0x45, 0x54, 0x48, 0x45, 0x49, 0x41]) // "ALETHEIA"
const EXPECTED_MAJOR_VERSION = 1

export class ParseErrorImpl extends Error {
  constructor(
    message: string,
    public readonly offset?: number
  ) {
    super(message)
    this.name = 'ParseError'
  }
}

class BinaryReader {
  private view: DataView
  private offset: number = 0

  constructor(private buffer: Uint8Array) {
    this.view = new DataView(buffer.buffer, buffer.byteOffset, buffer.byteLength)
  }

  get currentOffset(): number {
    return this.offset
  }

  readBytes(length: number): Uint8Array {
    if (this.offset + length > this.buffer.length) {
      throw new ParseErrorImpl(
        `Unexpected EOF: need ${length} bytes at offset ${this.offset}, but only ${
          this.buffer.length - this.offset
        } bytes available`,
        this.offset
      )
    }
    const bytes = this.buffer.slice(this.offset, this.offset + length)
    this.offset += length
    return bytes
  }

  readU8(): number {
    if (this.offset + 1 > this.buffer.length) {
      throw new ParseErrorImpl('Unexpected EOF reading u8', this.offset)
    }
    const value = this.view.getUint8(this.offset)
    this.offset += 1
    return value
  }

  readU16LE(): number {
    if (this.offset + 2 > this.buffer.length) {
      throw new ParseErrorImpl('Unexpected EOF reading u16', this.offset)
    }
    const value = this.view.getUint16(this.offset, true)
    this.offset += 2
    return value
  }

  readU32LE(): number {
    if (this.offset + 4 > this.buffer.length) {
      throw new ParseErrorImpl('Unexpected EOF reading u32', this.offset)
    }
    const value = this.view.getUint32(this.offset, true)
    this.offset += 4
    return value
  }

  readU64LE(): number {
    if (this.offset + 8 > this.buffer.length) {
      throw new ParseErrorImpl('Unexpected EOF reading u64', this.offset)
    }
    const value = Number(this.view.getBigUint64(this.offset, true))
    this.offset += 8
    return value
  }

  peek(length: number): Uint8Array {
    if (this.offset + length > this.buffer.length) {
      throw new ParseErrorImpl('Unexpected EOF in peek', this.offset)
    }
    return this.buffer.slice(this.offset, this.offset + length)
  }

  hasMore(): boolean {
    return this.offset < this.buffer.length
  }

  getRemainingBytes(): number {
    return this.buffer.length - this.offset
  }
}

function parseFlags(raw: Uint8Array): Flags {
  // Bit 0 = compression flag
  const isCompressed = (raw[0] & 0x01) !== 0
  return {
    raw,
    isCompressed,
  }
}

function parseCertificates(cborData: Uint8Array): Certificate[] {
  const decoded = decodeCBOR(cborData) as any[]

  return decoded.map((cert: any) => ({
    version: cert.version,
    serial: new Uint8Array(cert.serial),
    subject_id: cert.subject_id,
    subject_name: cert.subject_name,
    public_key: new Uint8Array(cert.public_key),
    issuer_id: cert.issuer_id,
    issued_at: cert.issued_at,
    is_ca: cert.is_ca,
    signature: new Uint8Array(cert.signature),
  }))
}

export function parseAletheiaFile(data: Uint8Array): AletheiaFile {
  const reader = new BinaryReader(data)

  // Magic bytes
  const magicStart = reader.currentOffset
  const magic = reader.readBytes(8)
  const magicRange: ByteRange = { start: magicStart, end: reader.currentOffset }

  for (let i = 0; i < 8; i++) {
    if (magic[i] !== MAGIC_BYTES[i]) {
      throw new ParseErrorImpl('Invalid magic bytes: not an Aletheia file', magicStart)
    }
  }

  // Version
  const versionStart = reader.currentOffset
  const versionMajor = reader.readU8()
  const versionMinor = reader.readU8()
  const versionRange: ByteRange = { start: versionStart, end: reader.currentOffset }

  if (versionMajor !== EXPECTED_MAJOR_VERSION) {
    throw new ParseErrorImpl(
      `Unsupported version: ${versionMajor}.${versionMinor} (expected ${EXPECTED_MAJOR_VERSION}.x)`,
      versionStart
    )
  }

  // Flags
  const flagsStart = reader.currentOffset
  const flagsRaw = reader.readBytes(2)
  const flags = parseFlags(flagsRaw)
  const flagsRange: ByteRange = { start: flagsStart, end: reader.currentOffset }

  // Header
  const headerLenStart = reader.currentOffset
  const headerLen = reader.readU32LE()
  const headerStart = reader.currentOffset
  const headerBytes = reader.readBytes(headerLen)
  const headerRange: ByteRange = { start: headerLenStart, end: reader.currentOffset }

  let header: Header
  try {
    const decoded = decodeCBOR(headerBytes) as any
    header = {
      creator_id: decoded.creator_id,
      signed_at: decoded.signed_at,
      content_type: decoded.content_type,
      original_name: decoded.original_name,
      description: decoded.description,
      custom: decoded.custom,
    }
  } catch (error) {
    throw new ParseErrorImpl(
      `Failed to decode header CBOR: ${error instanceof Error ? error.message : String(error)}`,
      headerStart
    )
  }

  // Payload
  const payloadLenStart = reader.currentOffset
  const payloadLen = reader.readU64LE()
  const payloadStart = reader.currentOffset

  if (payloadLen > Number.MAX_SAFE_INTEGER) {
    throw new ParseErrorImpl('Payload too large', payloadLenStart)
  }

  const payload = reader.readBytes(payloadLen)
  const payloadRange: ByteRange = { start: payloadLenStart, end: reader.currentOffset }

  // Certificate chain
  const certChainLenStart = reader.currentOffset
  const certChainLen = reader.readU32LE()
  const certChainStart = reader.currentOffset
  const certChainBytes = reader.readBytes(certChainLen)
  const certChainRange: ByteRange = { start: certChainLenStart, end: reader.currentOffset }

  let certificateChain: Certificate[]
  try {
    certificateChain = parseCertificates(certChainBytes)
  } catch (error) {
    throw new ParseErrorImpl(
      `Failed to decode certificate chain CBOR: ${error instanceof Error ? error.message : String(error)}`,
      certChainStart
    )
  }

  if (certificateChain.length === 0) {
    throw new ParseErrorImpl('Certificate chain is empty', certChainStart)
  }

  // Signature
  const signatureStart = reader.currentOffset
  if (reader.getRemainingBytes() !== 64) {
    throw new ParseErrorImpl(
      `Invalid signature length: expected 64 bytes, got ${reader.getRemainingBytes()}`,
      signatureStart
    )
  }
  const signature = reader.readBytes(64)
  const signatureRange: ByteRange = { start: signatureStart, end: reader.currentOffset }

  // Compute signature input: everything except the final 64-byte signature
  const signatureInput = data.slice(0, data.length - 64)

  return {
    rawBytes: data,
    magic: { value: magic, range: magicRange },
    version: { major: versionMajor, minor: versionMinor, range: versionRange },
    flags: { value: flags, range: flagsRange },
    header: { value: header, raw: headerBytes, range: headerRange },
    payload: { value: payload, range: payloadRange },
    certificateChain: { value: certificateChain, raw: certChainBytes, range: certChainRange },
    signature: { value: signature, range: signatureRange },
    signatureInput,
  }
}

export function decompressPayload(payload: Uint8Array, isCompressed: boolean): Uint8Array {
  if (!isCompressed) {
    return payload
  }

  // LZ4 decompression
  // The Rust code uses lz4_flex::compress_prepend_size
  // which prepends the uncompressed size as a little-endian u32

  // For browser, we'd need an LZ4 library
  // For now, throw an error - we can add lz4js if needed
  throw new Error('LZ4 decompression not yet implemented in browser')
}
