# Aletheia PKI Portal

A modern web-based Certificate Authority (CA) management system for the Aletheia project. The PKI Portal provides a comprehensive interface for managing digital certificates, trust bundles, revocations, and CA policies.

## Features

### Certificate Management
- **Root Certificate Management** - Import and manage root CA certificates
- **Intermediate Certificate Management** - Handle intermediate CA certificates
- **Certificate Issuance** - View all issued certificates with detailed information
- **Certificate Status** - Monitor certificate validity, expiration, and revocation status

### Security Operations
- **Revocation Management** - Revoke compromised certificates with detailed reasons
- **Trust Bundles** - Create and manage pre-configured trust bundles for distribution
- **Policy Management** - Configure and maintain CA policies
- **Audit Logs** - Comprehensive audit trail of all operations

### User Interface
- Clean, intuitive Material Design interface using Tailwind CSS
- Tabbed navigation for easy access to different management functions
- Real-time connection status to PKI backend
- Responsive design that works on desktop and tablet
- Color-coded status indicators (valid, expired, revoked)

## Getting Started

### Prerequisites
- Node.js 18+ 
- PKI Portal API running on `http://localhost:8080` (configurable via `.env`)

### Installation

1. Navigate to the aletheia-viewer directory:
```bash
cd aletheia-viewer
```

2. Install dependencies:
```bash
npm install
```

3. Create a `.env` file (copy from `.env.example` if needed):
```bash
cp .env.example .env
```

4. Update `.env` with your PKI Portal API endpoint:
```
VITE_API_URL=http://localhost:8080
```

### Development

Start the development server:
```bash
npm run dev
```

Access the portal at:
- **File Viewer**: `http://localhost:5173/?mode=viewer`
- **PKI Portal**: `http://localhost:5173/?mode=portal`

### Production Build

Build for production:
```bash
npm run build
```

Preview production build:
```bash
npm run preview
```

## Architecture

### Component Structure

```
src/components/
├── PKIPortalView.vue              # Main portal container
└── pki-portal/
    ├── RootsManagement.vue         # Root CA management
    ├── IntermediatesManagement.vue # Intermediate CA management
    ├── CertificateManagement.vue   # Issued certificates view
    ├── RevocationManagement.vue    # Certificate revocation interface
    ├── TrustBundlesManagement.vue  # Trust bundle creation/management
    ├── PolicyManagement.vue        # CA policy configuration
    └── AuditLogs.vue              # Audit log viewer
```

### API Integration

The portal communicates with the PKI Portal backend via REST API:

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/health` | GET | Backend health check |
| `/roots` | GET, POST | Root certificate management |
| `/intermediates` | GET, POST | Intermediate certificate management |
| `/certificates` | GET | View issued certificates |
| `/revocations` | GET, POST | Certificate revocation |
| `/trust-bundles` | GET, POST | Trust bundle management |
| `/policy` | GET, POST | Policy management |
| `/audit-logs` | GET | Audit log retrieval |

## Features by Tab

### 🔐 Root Certificates
- Upload and manage root CA certificates
- View certificate details (subject, issuer, fingerprint, validity dates)
- Monitor certificate expiration

### 🔗 Intermediates
- Manage intermediate CA certificates
- View certificate chain relationships
- Track certificate validity

### 📜 Certificates
- View all issued certificates
- Check certificate status (valid, expired, revoked)
- Access certificate details

### ⛔ Revocations
- Revoke compromised certificates
- Specify revocation reasons
- Maintain revocation list

### 📦 Trust Bundles
- Create pre-configured trust bundles
- Bundle multiple certificates for distribution
- Organize certificates by purpose

### ⚙️ Policy
- Define and manage CA policies
- Configure policy parameters
- Enforce security policies

### 📋 Audit Logs
- View comprehensive audit trail
- Filter operations by type and date
- Track user actions and changes

## Environment Variables

Create a `.env` file in the `aletheia-viewer` directory:

```env
# PKI Portal API base URL
VITE_API_URL=http://localhost:8080

# Application environment
VITE_ENV=development
```

## Browser Support

- Chrome/Edge 90+
- Firefox 88+
- Safari 14+

## Security Considerations

1. **HTTPS in Production** - Always use HTTPS in production environments
2. **CORS Configuration** - Ensure proper CORS settings on the backend
3. **Authentication** - Consider implementing authentication in future versions
4. **Input Validation** - All inputs are validated before sending to the backend
5. **Error Handling** - Sensitive errors are caught and logged appropriately

## Troubleshooting

### Connection Issues
If the portal shows "Unable to Connect":
1. Verify the PKI Portal API is running on the configured URL
2. Check the `VITE_API_URL` environment variable
3. Ensure CORS is enabled on the backend
4. Check browser console for detailed error messages

### Build Issues
If you encounter build issues:
1. Clear `node_modules` and reinstall: `rm -rf node_modules && npm install`
2. Clear Vite cache: `rm -rf dist`
3. Check Node.js version: `node --version` (should be 18+)

## Development Workflow

1. **Create a new management feature**:
   - Add component to `src/components/pki-portal/`
   - Import in `PKIPortalView.vue`
   - Add tab to navigation

2. **Add API endpoints**:
   - Update the component's fetch calls
   - Ensure proper error handling
   - Test with the running backend

3. **Test changes**:
   - Use dev server: `npm run dev`
   - Test all CRUD operations
   - Verify error handling

## Future Enhancements

- User authentication and authorization
- Certificate details modal/drawer
- Batch operations
- Search and filtering
- Export functionality
- Certificate lifecycle visualization
- Real-time notifications for events
- Multi-tenant support

## License

MIT
