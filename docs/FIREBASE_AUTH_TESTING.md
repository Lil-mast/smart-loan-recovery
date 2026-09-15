# Firebase Auth Testing Guide

## 🔧 Setup Steps

### 1. Configure Your Environment Files

#### `.env` file (already configured ✅)
```env
SERVER_PORT=8080
SESSION_SECRET=a3f5c8e9d2b1a4f7e6c3d8b5a9f2e1c4d7b6a3f8e5c2d9b4a7f6e3c8d5b2a1
```

#### `.env.firebase` file (YOU MUST UPDATE)
```env
# Get these from Firebase Console > Project Settings > General
FIREBASE_PROJECT_ID=your-actual-project-id
FIREBASE_API_KEY=your-actual-api-key
FIREBASE_AUTH_DOMAIN=your-project-id.firebaseapp.com

# Generate with: openssl rand -hex 32
JWT_SECRET=your-generated-secret-here
```

### 2. Get Your Firebase Credentials

1. Go to [Firebase Console](https://console.firebase.google.com/)
2. Create a project (or use existing)
3. Click ⚙️ **Project Settings** > **General**
4. Copy:
   - **Project ID** (e.g., `my-loan-app-12345`)
   - **Web API Key** (e.g., `AIzaSyB...`)
5. Paste into `.env.firebase`

### 3. Enable Authentication Methods

In Firebase Console:
1. Go to **Build** > **Authentication**
2. Click **Get Started**
3. Enable **Email/Password** (Native provider)
4. Enable **Google** (under Additional providers)

---

## 🧪 Postman Testing Collection

### Base URL
```
http://localhost:8080
```

---

### 1. Health Check

**GET** `/`

```json
Response:
{
  "message": "Smart Loan Recovery API is running!",
  "version": "1.0.0",
  "features": {
    "firebase_auth": true,
    "jwt_tokens": true,
    "role_based_access": true,
    "google_signin": true
  }
}
```

---

### 2. Register New User

**POST** `/auth/register`

**Headers:**
```
Content-Type: application/json
```

**Body:**
```json
{
  "email": "test@example.com",
  "password": "SecurePass123!",
  "name": "Test User",
  "role": "borrower"
}
```

**Expected Response (201 Created):**
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIs...",
  "refresh_token": "eyJhbGciOiJIUzI1NiIs...",
  "token_type": "Bearer",
  "expires_in": 86400,
  "user": {
    "uid": "firebase-user-id",
    "email": "test@example.com",
    "email_verified": false,
    "name": "Test User",
    "role": "borrower",
    "photo_url": null,
    "local_user_id": "local-uuid"
  }
}
```

---

### 3. Login

**POST** `/auth/login`

**Body:**
```json
{
  "email": "test@example.com",
  "password": "SecurePass123!"
}
```

**Expected Response (200 OK):**
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIs...",
  "refresh_token": "eyJhbGciOiJIUzI1NiIs...",
  "token_type": "Bearer",
  "expires_in": 86400,
  "user": {
    "uid": "...",
    "email": "test@example.com",
    "email_verified": false,
    "name": "test",
    "role": "borrower",
    "local_user_id": "..."
  }
}
```

---

### 4. Access Protected Route (with JWT)

**GET** `/api/users`

**Headers:**
```
Authorization: Bearer eyJhbGciOiJIUzI1NiIs... (your access_token)
```

**Expected Response (200 OK):**
```json
[
  {
    "id": "abc1",
    "name": "Demo Lender",
    "role": "Lender",
    "email": "lender@example.com"
  }
]
```

---

### 5. Verify Token

**POST** `/auth/verify`

**Headers:**
```
Authorization: Bearer eyJhbGciOiJIUzI1NiIs...
```

**Expected Response:**
```json
{
  "valid": true,
  "user": {
    "uid": "...",
    "email": "test@example.com",
    "role": "borrower"
  }
}
```

---

### 6. Refresh Token

**POST** `/auth/refresh`

**Body:**
```json
{
  "refresh_token": "eyJhbGciOiJIUzI1NiIs..."
}
```

**Expected Response:**
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIs... (new)",
  "refresh_token": "eyJhbGciOiJIUzI1NiIs... (new)",
  "expires_in": 86400
}
```

---

### 7. Logout

**POST** `/auth/logout`

**Headers:**
```
Authorization: Bearer eyJhbGciOiJIUzI1NiIs...
```

**Body:**
```json
{
  "refresh_token": "eyJhbGciOiJIUzI1NiIs..."
}
```

**Expected Response:**
```json
{
  "message": "Logged out successfully"
}
```

---

### 8. Get Current User

**GET** `/auth/me`

**Headers:**
```
Authorization: Bearer eyJhbGciOiJIUzI1NiIs...
```

---

### 9. Google Sign-In

**POST** `/auth/google`

**Body:**
```json
{
  "id_token": "google-id-token-from-frontend"
}
```

---

## 🔑 Postman Environment Setup

Create a Postman Environment with these variables:

| Variable | Initial Value | Current Value |
|----------|---------------|---------------|
| `base_url` | `http://localhost:8080` | `http://localhost:8080` |
| `access_token` | *empty* | (auto-filled after login) |
| `refresh_token` | *empty* | (auto-filled after login) |

### Postman Tests (to auto-save tokens)

Add this to your **Login** request under **Tests** tab:

```javascript
// Parse response
var jsonData = pm.response.json();

// Save tokens to environment
pm.environment.set("access_token", jsonData.access_token);
pm.environment.set("refresh_token", jsonData.refresh_token);

console.log("✅ Access token saved");
console.log("✅ Refresh token saved");
```

---

## 🚀 Running the Server

```bash
# 1. Make sure your .env.firebase has real credentials
# 2. Build and run
cargo run server

# Server will start on http://localhost:8080
```

---

## 📋 Testing Checklist

- [ ] Server starts without errors
- [ ] Health check returns 200
- [ ] Can register new user
- [ ] Can login with credentials
- [ ] Protected routes reject requests without token (401)
- [ ] Protected routes accept valid token (200)
- [ ] Token refresh works
- [ ] Logout invalidates token
- [ ] Google Sign-In works (with valid Google token)

---

## 🐛 Common Issues

### "Firebase authentication initialization failed"
- Check `.env.firebase` has valid credentials
- Verify `FIREBASE_PROJECT_ID` and `FIREBASE_API_KEY` are correct

### "Invalid email or password" (401)
- User doesn't exist - register first
- Wrong password

### "Unauthorized" (401) on protected routes
- Missing `Authorization` header
- Token expired - use refresh endpoint
- Token blacklisted (after logout)

### "Token has been revoked" (401)
- User logged out
- Use login to get new tokens

---

## 📝 Quick Test Script (curl)

```bash
# 1. Register
curl -X POST http://localhost:8080/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"test@test.com","password":"test1234","name":"Test User","role":"borrower"}'

# 2. Login
curl -X POST http://localhost:8080/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"test@test.com","password":"test1234"}'

# 3. Access protected route (replace TOKEN)
curl http://localhost:8080/api/users \
  -H "Authorization: Bearer YOUR_TOKEN_HERE"
```
