// Extract CA public key from test-file.alx
import { readFileSync, writeFileSync } from 'fs';
import init, { parse_aletheia_file } from './src/lib/wasm-pkg/aletheia.js';

await init();

const alxFile = readFileSync('test-file.alx');
const parsed = parse_aletheia_file(alxFile);

// Get the root certificate (last in chain)
const rootCert = parsed.certificateChain[parsed.certificateChain.length - 1];

if (rootCert && rootCert.publicKey) {
  // Convert to hex
  const pubKeyHex = Buffer.from(rootCert.publicKey).toString('hex');

  // Save as .pub file
  writeFileSync('ca.pub', pubKeyHex);

  console.log('✅ Created ca.pub');
  console.log('Root CA:', rootCert.subjectName, `(${rootCert.subjectId})`);
  console.log('Public key:', pubKeyHex);
} else {
  console.error('Could not extract public key');
}
