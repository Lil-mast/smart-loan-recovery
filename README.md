# LendWise Recovery

[![Deployed on Fly.io](https://img.shields.io/badge/Deployed%20on-Fly.io-blue)](https://lendwise-recovery.fly.dev/)
[![Rust](https://img.shields.io/badge/Rust-1.92-orange)](https://www.rust-lang.org/)
[![Docker](https://img.shields.io/badge/Docker-Ready-blue)](https://www.docker.com/)
[![Rust Africa Hackathon](https://img.shields.io/badge/Built%20for-%23RustAfricaHackathon-brightgreen)](https://rust-africa-hackathon.io/)

🚀 **Built for #RustAfricaHackathon** 🚀

An innovative AI-enhanced loan recovery system built with Rust, featuring a secure web API, authentication, and intelligent recovery recommendations. This project demonstrates modern financial technology solutions optimized for African fintech challenges.

**Objectives, traceability to code, and a short comparison** to other recovery approaches: [docs/OBJECTIVES_TRACEABILITY.md](docs/OBJECTIVES_TRACEABILITY.md).

## 🌐 Live Demo

**Production URL**: https://lendwise-recovery.fly.dev/

## ✨ Features

### 🔐 **Authentication & Security**
- User registration and login system
- Session-based authentication with secure cookies
- Role-based access control (Borrowers & Lenders)
- Password-less authentication (name-based for demo)

### 🏦 **Loan Management**
- Complete loan lifecycle tracking
- Real-time loan status monitoring (Active, Overdue, Defaulted, Repaid)
- Comprehensive loan data with repayment schedules
- Principal, interest rate, and duration tracking

### 🤖 **AI-Powered Recovery**
- Intelligent recovery action recommendations using smart algorithms
- Risk assessment based on loan status and history
- Automated overdue loan detection
- Rule-based recovery strategies optimized for maximum recovery rates

### 💾 **Data Persistence**
- SQLite database with automatic schema management
- JSON backup/restore functionality for data resilience
- UUID-based entity identification
- Thread-safe database operations

### 🐳 **Containerization & Deployment**
- Docker containerization for easy deployment across environments
- Multi-stage Docker builds for optimized images
- Production-ready configuration
- One-click deployment to Fly.io

### 🧪 **Testing & Quality**
- Comprehensive integration tests for reliability
- Automated testing pipeline
- Custom error handling with detailed error types
- Structured logging throughout the system

## 🚀 Quick Start

### Prerequisites
- Rust 1.92 or later
- Docker (optional, for containerized deployment)

### Local Development

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd lendwise-recovery
   ```

2. **Run the backend** (from the repository root so `frontend/` is found for `/app/`)
   ```bash
   cargo run
   ```
   - API root: `http://127.0.0.1:3000` (JSON status at `GET /`)
   - **Web UI (recommended):** `http://127.0.0.1:3000/app/` — same origin as the API
   - Optional: `RUST_LOG=info cargo run` for request logs
   - Ensure `.env` does not set `SESSION_SECRET` to an empty value (or omit it to use the built-in dev default)

3. **Run with Docker**
   ```bash
   docker build -t smart-loan-recovery .
   docker run -p 3000:3000 smart-loan-recovery
   ```

## 📡 API Endpoints

### Authentication
- `POST /users` - Register a new user
- `POST /login` - Login with user credentials
- `POST /logout` - Logout current user
- `GET /me` - Get current user information

### Loans
- `GET /loans` - List all loans (authenticated)
- `POST /loans` - Create a new loan (lenders only)

### Recovery
- `POST /overdues` - Flag overdue loans (admin)
- `POST /recommend/{loan_id}` - Get recovery recommendation

### System
- `GET /` - API information and available endpoints

## 🔧 Configuration

The application uses environment variables for configuration:

```bash
# Server Configuration
SERVER_HOST=0.0.0.0          # Host to bind to (0.0.0.0 for all interfaces)
SERVER_PORT=3000             # Port to listen on

# Database
DATABASE_URL=loans.db        # SQLite database file path

# Security
SESSION_SECRET=your-secret-key-here  # Session encryption key
```

## 🏗️ Architecture

```
src/
├── main.rs          # Application entry point
├── api.rs           # Web API routes and handlers
├── db.rs            # Database operations
├── user.rs          # User management
├── loan.rs          # Loan operations
├── recovery.rs      # AI recovery engine
├── models.rs        # Data structures
├── config.rs        # Configuration management
├── error.rs         # Error handling
└── lib.rs           # Library exports
```

### Key Components

- **Actix Web**: High-performance web framework
- **SQLite + Rusqlite**: Embedded database
- **Actix Identity**: Session management
- **Actix Session**: Secure cookie sessions
- **Serde**: Serialization/deserialization
- **UUID**: Unique identifier generation
- **Chrono**: Date/time handling

## 🧪 Testing

Run the test suite:

```bash
cargo test
```

Run integration tests:

```bash
cargo test --test integration_tests
```

## 🐳 Docker Deployment

### Build the Image
```bash
docker build -t smart-loan-recovery .
```

### Run Locally
```bash
docker run -p 3000:3000 smart-loan-recovery
```

### Deploy to Fly.io
```bash
fly launch
fly deploy
```

## 📖 User Guide 

### Step-by-Step: Register for a Loan and Get Recovery Recommendations

#### Step 1: Register as a User

**API Call:**
```bash
curl -X POST https://smart-loan-recovery.fly.dev/users \
  -H "Content-Type: application/json" \
  -d '{
    "name": "John Doe",
    "role": "borrower"
  }'
```

**Response:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000"
}
```

#### Step 2: Login

**API Call:**
```bash
curl -X POST https://smart-loan-recovery.fly.dev/login \
  -H "Content-Type: application/json" \
  -d '{
    "name": "John Doe"
  }'
```

**Response:**
```json
{
  "message": "Login successful",
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "role": "borrower"
}
```

#### Step 3: Register as a Lender (if needed)

**API Call:**
```bash
curl -X POST https://smart-loan-recovery.fly.dev/users \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Bank Corp",
    "role": "lender"
  }'
```

#### Step 4: Create a Loan (as Lender)

**API Call:**
```bash
curl -X POST https://smart-loan-recovery.fly.dev/loans \
  -H "Content-Type: application/json" \
  -d '{
    "borrower_id": "550e8400-e29b-41d4-a716-446655440000",
    "lender_id": "660e8400-e29b-41d4-a716-446655440001",
    "principal": 10000.00,
    "interest_rate": 5.5,
    "months": 12
  }'
```

**Response:**
```json
{
  "id": "770e8400-e29b-41d4-a716-446655440002"
}
```

#### Step 5: Check Loan Status

**API Call:**
```bash
curl -X GET https://smart-loan-recovery.fly.dev/loans
```

#### Step 6: Flag Overdue Loans (Admin Function)

**API Call:**
```bash
curl -X POST https://smart-loan-recovery.fly.dev/overdues
```

**Response:**
```json
{
  "message": "Overdue loans flagged successfully",
  "flagged_count": 1
}
```

#### Step 7: Get Recovery Recommendation

**API Call:**
```bash
curl -X POST https://smart-loan-recovery.fly.dev/recommend/770e8400-e29b-41d4-a716-446655440002
```

**Response:**
```json
{
  "loan_id": "770e8400-e29b-41d4-a716-446655440002",
  "risk_score": 8.5,
  "recommendation": "immediate_contact",
  "actions": [
    "Send payment reminder email",
    "Schedule phone call within 24 hours",
    "Review loan terms and payment history"
  ]
}
```

## 🔒 Security Features

- **Session Management**: Secure cookie-based sessions
- **Authentication**: Required for sensitive operations
- **Authorization**: Role-based access control
- **Input Validation**: Comprehensive request validation
- **Error Handling**: Secure error responses without data leakage

## 📊 Recovery Actions

The system provides intelligent recovery recommendations:

- **Low Risk (0-3)**: `monitor` - Regular monitoring
- **Medium Risk (3-7)**: `follow_up` - Payment reminders
- **High Risk (7-10)**: `immediate_contact` - Urgent intervention

## 🤝 Contributing

**Hackathon Participants**: We welcome contributions! Follow these steps:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes and test thoroughly
4. Add integration tests for new features
5. Submit a pull request with a clear description

**Issues & Ideas**: Have suggestions? Open an issue on GitHub!

## 📄 License

This project is licensed under the APACHE License - see the LICENSE file for details.

## 🌟 Built with ❤️ for #RustAfricaHackathon

Special thanks to the Rust and African tech communities for the inspiration and support.

## 🙏 Acknowledgments

- Built with Rust and Actix Web
- SQLite for data persistence
- Docker for containerization
- Fly.io for hassle-free hosting
- The Rust community for amazing tools and frameworks