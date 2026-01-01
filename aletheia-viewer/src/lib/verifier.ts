// @ts-nocheck
/**
 * Aletheia file verification logic
 *
 * Verification steps:
 * 1. Parse file structure
 * 2. Decode CBOR (header + certificates)
 * 3. Verify certificate chain signatures
 * 4. Verify root is trusted
 * 5. Verify file signature
 */

import * as ed25519 from '@noble/ed25519'
import { encode as encodeCBOR } from 'cbor-x'
import type { AletheiaFile, Certificate, VerificationStep, VerificationResult } from '../types/aletheia'

/**
 * Compute the signable data for a certificate
 * This is the CBOR encoding of the certificate with signature field omitted
 */
function getCertificateSignableData(cert: Certificate): Uint8Array {
  // Create a copy without the signature field
  const signable = {
    version: cert.version,
    serial: cert.serial,
    subject_id: cert.subject_id,
    subject_name: cert.subject_name,
    public_key: cert.public_key,
    issuer_id: cert.issuer_id,
    issued_at: cert.issued_at,
    is_ca: cert.is_ca,
  }

  return encodeCBOR(signable)
}

/**
 * Verify a single certificate's signature using the issuer's public key
 */
async function verifyCertificateSignature(
  cert: Certificate,
  issuerPublicKey: Uint8Array
): Promise<{ valid: boolean; error?: string }> {
  try {
    const signableData = getCertificateSignableData(cert)
    const valid = await ed25519.verify(cert.signature, signableData, issuerPublicKey)
    return { valid }
  } catch (error) {
    return {
      valid: false,
      error: error instanceof Error ? error.message : 'Signature verification failed',
    }
  }
}

/**
 * Verify the certificate chain
 *
 * Chain order: [creator_cert, intermediate_certs..., root_cert]
 * Each certificate must be signed by the next one in the chain
 * Root must be self-signed
 */
async function verifyCertificateChain(
  chain: Certificate[],
  trustedRoots: Uint8Array[]
): Promise<{ valid: boolean; error?: string }> {
  if (chain.length === 0) {
    return { valid: false, error: 'Certificate chain is empty' }
  }

  // Verify each certificate in the chain
  for (let i = 0; i < chain.length; i++) {
    const cert = chain[i]

    // Get the issuer's public key
    let issuerKey: Uint8Array

    if (i + 1 < chain.length) {
      // Issuer is the next certificate in the chain
      const issuer = chain[i + 1]

      // Verify issuer is allowed to issue certificates
      if (!issuer.is_ca) {
        return {
          valid: false,
          error: `Certificate '${issuer.subject_id}' is not a CA but issued '${cert.subject_id}'`,
        }
      }

      // Verify issuer ID matches
      if (cert.issuer_id !== issuer.subject_id) {
        return {
          valid: false,
          error: `Issuer ID mismatch: cert says '${cert.issuer_id}', chain has '${issuer.subject_id}'`,
        }
      }

      issuerKey = issuer.public_key
    } else {
      // This is the root certificate - must be self-signed
      if (cert.issuer_id !== cert.subject_id) {
        return {
          valid: false,
          error: 'Root certificate is not self-signed',
        }
      }

      // Root must be a CA
      if (!cert.is_ca) {
        return {
          valid: false,
          error: 'Root certificate is not marked as CA',
        }
      }

      // Verify root is trusted
      const isTrusted = trustedRoots.some((trustedKey) => arraysEqual(trustedKey, cert.public_key))

      if (!isTrusted) {
        return {
          valid: false,
          error: 'Root certificate is not in trusted roots',
        }
      }

      issuerKey = cert.public_key // Self-signed
    }

    // Verify this certificate's signature
    const result = await verifyCertificateSignature(cert, issuerKey)
    if (!result.valid) {
      return {
        valid: false,
        error: `Certificate '${cert.subject_id}' signature verification failed: ${result.error}`,
      }
    }
  }

  return { valid: true }
}

/**
 * Verify the file signature
 */
async function verifyFileSignature(file: AletheiaFile): Promise<{ valid: boolean; error?: string }> {
  try {
    const creatorCert = file.certificateChain.value[0]
    const creatorPublicKey = creatorCert.public_key
    const signature = file.signature.value
    const signatureInput = file.signatureInput

    const valid = await ed25519.verify(signature, signatureInput, creatorPublicKey)

    return { valid }
  } catch (error) {
    return {
      valid: false,
      error: error instanceof Error ? error.message : 'File signature verification failed',
    }
  }
}

/**
 * Verify header creator_id matches certificate
 */
function verifyCreatorId(file: AletheiaFile): { valid: boolean; error?: string } {
  const creatorCert = file.certificateChain.value[0]
  if (file.header.value.creator_id !== creatorCert.subject_id) {
    return {
      valid: false,
      error: `Creator ID mismatch: header says '${file.header.value.creator_id}', certificate says '${creatorCert.subject_id}'`,
    }
  }
  return { valid: true }
}

/**
 * Perform full verification of an Aletheia file
 */
export async function verifyAletheiaFile(
  file: AletheiaFile,
  trustedRoots: Uint8Array[]
): Promise<VerificationResult> {
  const steps: VerificationStep[] = [
    { id: 'parse', label: 'Parse file structure', status: 'success' },
    { id: 'cbor', label: 'Decode CBOR (header + certificates)', status: 'success' },
    { id: 'creator_id', label: 'Verify creator ID matches certificate', status: 'pending' },
    { id: 'cert_chain', label: 'Verify certificate chain signatures', status: 'pending' },
    { id: 'trusted_root', label: 'Verify root is trusted', status: 'pending' },
    { id: 'file_sig', label: 'Verify file signature', status: 'pending' },
  ]

  let isValid = true

  // Parsing and CBOR already succeeded if we got here
  // (otherwise parseAletheiaFile would have thrown)

  // Step 3: Verify creator ID
  const creatorIdResult = verifyCreatorId(file)
  const creatorIdStep = steps.find((s) => s.id === 'creator_id')!
  if (creatorIdResult.valid) {
    creatorIdStep.status = 'success'
  } else {
    creatorIdStep.status = 'error'
    creatorIdStep.error = creatorIdResult.error
    isValid = false
  }

  // Step 4: Verify certificate chain (includes root trust check)
  const chainResult = await verifyCertificateChain(file.certificateChain.value, trustedRoots)
  const chainStep = steps.find((s) => s.id === 'cert_chain')!
  const rootStep = steps.find((s) => s.id === 'trusted_root')!

  if (chainResult.valid) {
    chainStep.status = 'success'
    rootStep.status = 'success'
  } else {
    // Determine which step failed
    if (chainResult.error?.includes('not in trusted roots')) {
      chainStep.status = 'success'
      rootStep.status = 'error'
      rootStep.error = chainResult.error
    } else {
      chainStep.status = 'error'
      chainStep.error = chainResult.error
      rootStep.status = 'pending'
    }
    isValid = false
  }

  // Step 5: Verify file signature (only if chain is valid)
  if (chainResult.valid) {
    const sigResult = await verifyFileSignature(file)
    const sigStep = steps.find((s) => s.id === 'file_sig')!

    if (sigResult.valid) {
      sigStep.status = 'success'
    } else {
      sigStep.status = 'error'
      sigStep.error = sigResult.error
      isValid = false
    }
  }

  return {
    isValid,
    steps,
  }
}

/**
 * Extract trusted root public keys from certificate files
 */
export async function extractTrustedRoots(certificateFiles: File[]): Promise<Uint8Array[]> {
  const roots: Uint8Array[] = []

  for (const file of certificateFiles) {
    try {
      const arrayBuffer = await file.arrayBuffer()
      const bytes = new Uint8Array(arrayBuffer)

      // Try to parse as raw public key (32 bytes)
      if (bytes.length === 32) {
        roots.push(bytes)
        continue
      }

      // Try to parse as hex-encoded
      const text = new TextDecoder().decode(bytes).trim()
      if (/^[0-9a-fA-F]{64}$/.test(text)) {
        const hexBytes = new Uint8Array(32)
        for (let i = 0; i < 32; i++) {
          hexBytes[i] = parseInt(text.substr(i * 2, 2), 16)
        }
        roots.push(hexBytes)
        continue
      }

      // If it's a full certificate, extract the public key
      // (Would need to parse CBOR certificate format)
      console.warn(`Skipping file '${file.name}': not a recognized format`)
    } catch (error) {
      console.error(`Failed to process file '${file.name}':`, error)
    }
  }

  return roots
}

function arraysEqual(a: Uint8Array, b: Uint8Array): boolean {
  if (a.length !== b.length) return false
  for (let i = 0; i < a.length; i++) {
    if (a[i] !== b[i]) return false
  }
  return true
}
