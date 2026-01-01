// Quick script to extract public key from CA private key
const fs = require('fs');
const crypto = require('crypto');

// Read private key
const privKeyHex = fs.readFileSync('ca.key', 'utf8').trim();
const privKey = Buffer.from(privKeyHex, 'hex');

// Ed25519: public key is derived from private key
// In Ed25519, the public key is derived using SHA-512 and curve operations
// For simplicity, we'll parse the certificate to get the public key

// Read the certificate and parse CBOR to extract public key
const cert = fs.readFileSync('ca.cert');
const cbor = require('cbor');

cbor.decodeFirst(cert, (err, obj) => {
  if (err) {
    console.error('Error parsing certificate:', err);
    process.exit(1);
  }

  const publicKey = obj.public_key;
  if (publicKey) {
    const pubKeyHex = Buffer.from(publicKey).toString('hex');
    fs.writeFileSync('ca.pub', pubKeyHex);
    console.log('✅ Created ca.pub');
    console.log('Public key:', pubKeyHex);
  } else {
    console.error('No public_key found in certificate');
  }
});
