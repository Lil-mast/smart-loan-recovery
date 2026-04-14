/**
 * Firebase Authentication Module
 * Handles Google Sign-In, Email/Password auth, and token management
 * Uses httpOnly cookies for secure JWT storage
 */

// API base URL
const API_BASE = window.location.origin.includes('localhost') || window.location.origin.includes('127.0.0.1')
    ? 'http://127.0.0.1:3000'
    : window.location.origin;

// Firebase configuration (loaded from backend)
let firebaseConfig = null;
let firebaseApp = null;
let firebaseAuth = null;

/**
 * Initialize Firebase Authentication
 * Loads config from backend and initializes Firebase
 */
async function initFirebaseAuth() {
    try {
        // Fetch Firebase config from backend
        const response = await fetch(`${API_BASE}/auth/config`, {
            credentials: 'include'
        });
        
        if (!response.ok) {
            throw new Error('Failed to load Firebase configuration');
        }
        
        firebaseConfig = await response.json();
        
        // Initialize Firebase
        firebaseApp = firebase.initializeApp(firebaseConfig);
        firebaseAuth = firebase.auth();
        
        // Set up auth state listener
        firebaseAuth.onAuthStateChanged(handleAuthStateChanged);
        
        console.log('✅ Firebase Auth initialized');
        return true;
    } catch (error) {
        console.error('❌ Firebase Auth initialization failed:', error);
        return false;
    }
}

/**
 * Handle authentication state changes
 */
function handleAuthStateChanged(user) {
    if (user) {
        console.log('User is signed in:', user.email);
        updateUIForAuthenticatedUser(user);
    } else {
        console.log('User is signed out');
        updateUIForGuest();
    }
}

/**
 * Sign in with Google
 * Opens Google Sign-In popup
 */
async function signInWithGoogle() {
    try {
        const provider = new firebase.auth.GoogleAuthProvider();
        provider.addScope('email');
        provider.addScope('profile');
        
        const result = await firebaseAuth.signInWithPopup(provider);
        const user = result.user;
        const idToken = await user.getIdToken();
        
        // Send token to backend for verification
        const response = await fetch(`${API_BASE}/auth/google`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            credentials: 'include',
            body: JSON.stringify({ id_token: idToken })
        });
        
        if (response.ok) {
            const data = await response.json();
            console.log('✅ Google Sign-In successful');
            handleLoginSuccess(data);
            return data;
        } else {
            const error = await response.json();
            throw new Error(error.error || 'Google Sign-In failed');
        }
    } catch (error) {
        console.error('❌ Google Sign-In error:', error);
        throw error;
    }
}

/**
 * Sign in with email and password
 */
async function signInWithEmail(email, password) {
    try {
        const response = await fetch(`${API_BASE}/auth/login`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            credentials: 'include',
            body: JSON.stringify({ email, password })
        });
        
        if (response.ok) {
            const data = await response.json();
            console.log('✅ Login successful');
            handleLoginSuccess(data);
            return data;
        } else {
            const error = await response.json();
            throw new Error(error.error || 'Login failed');
        }
    } catch (error) {
        console.error('❌ Login error:', error);
        throw error;
    }
}

/**
 * Register with email and password
 */
async function registerWithEmail(email, password, name, role = 'borrower') {
    try {
        const response = await fetch(`${API_BASE}/auth/register`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            credentials: 'include',
            body: JSON.stringify({ email, password, name, role })
        });
        
        if (response.ok) {
            const data = await response.json();
            console.log('✅ Registration successful');
            handleLoginSuccess(data);
            return data;
        } else {
            const error = await response.json();
            throw new Error(error.error || 'Registration failed');
        }
    } catch (error) {
        console.error('❌ Registration error:', error);
        throw error;
    }
}

/**
 * Logout user
 */
async function logout() {
    try {
        // Sign out from Firebase
        if (firebaseAuth) {
            await firebaseAuth.signOut();
        }
        
        // Call backend logout (clears cookies)
        await fetch(`${API_BASE}/auth/logout`, {
            method: 'POST',
            credentials: 'include'
        });
        
        console.log('✅ Logout successful');
        
        // Clear local state
        localStorage.removeItem('user');
        localStorage.removeItem('lenderSession');
        
        // Redirect to home
        window.location.href = 'index.html';
    } catch (error) {
        console.error('❌ Logout error:', error);
    }
}

/**
 * Get current user profile
 */
async function getCurrentUser() {
    try {
        const response = await fetch(`${API_BASE}/auth/me`, {
            credentials: 'include'
        });
        
        if (response.ok) {
            return await response.json();
        }
        return null;
    } catch (error) {
        console.error('❌ Get current user error:', error);
        return null;
    }
}

/**
 * Refresh access token using refresh token cookie
 */
async function refreshToken() {
    try {
        const response = await fetch(`${API_BASE}/auth/refresh`, {
            method: 'POST',
            credentials: 'include'
        });
        
        if (response.ok) {
            const data = await response.json();
            console.log('✅ Token refreshed');
            return data;
        } else {
            throw new Error('Token refresh failed');
        }
    } catch (error) {
        console.error('❌ Token refresh error:', error);
        // Redirect to login if refresh fails
        window.location.href = 'index.html';
    }
}

/**
 * Handle successful login
 */
function handleLoginSuccess(data) {
    const user = data.user;
    
    // Store user info (not tokens - they're in httpOnly cookies)
    localStorage.setItem('user', JSON.stringify(user));
    
    // Redirect based on role
    if (user.role === 'borrower') {
        window.location.href = `borrowers.html?user_id=${user.local_user_id}`;
    } else if (user.role === 'lender' || user.role === 'admin') {
        localStorage.setItem('lenderSession', JSON.stringify(user));
        window.location.href = 'lenders.html';
    }
}

/**
 * Update UI for authenticated user
 */
function updateUIForAuthenticatedUser(user) {
    // Show logout button, hide login/register buttons
    const navGuestButtons = document.getElementById('navGuestButtons');
    const navLogoutBtn = document.getElementById('navLogoutBtn');
    
    if (navGuestButtons) navGuestButtons.classList.add('hidden');
    if (navLogoutBtn) {
        navLogoutBtn.classList.remove('hidden');
        navLogoutBtn.onclick = logout;
    }
}

/**
 * Update UI for guest user
 */
function updateUIForGuest() {
    // Show login/register buttons, hide logout button
    const navGuestButtons = document.getElementById('navGuestButtons');
    const navLogoutBtn = document.getElementById('navLogoutBtn');
    
    if (navGuestButtons) navGuestButtons.classList.remove('hidden');
    if (navLogoutBtn) navLogoutBtn.classList.add('hidden');
}

/**
 * Check if user is authenticated
 */
async function isAuthenticated() {
    const user = await getCurrentUser();
    return user !== null;
}

/**
 * API call wrapper with automatic token refresh
 */
async function apiCall(url, options = {}) {
    // Ensure credentials are included
    options.credentials = 'include';
    
    try {
        let response = await fetch(url, options);
        
        // If 401, try to refresh token
        if (response.status === 401) {
            await refreshToken();
            response = await fetch(url, options);
        }
        
        return response;
    } catch (error) {
        console.error('❌ API call error:', error);
        throw error;
    }
}

// Initialize on page load
document.addEventListener('DOMContentLoaded', async () => {
    await initFirebaseAuth();
    
    // Check current auth state
    const user = await getCurrentUser();
    if (user) {
        updateUIForAuthenticatedUser(user);
    } else {
        updateUIForGuest();
    }
});

// Export functions for global use
window.FirebaseAuth = {
    signInWithGoogle,
    signInWithEmail,
    registerWithEmail,
    logout,
    getCurrentUser,
    isAuthenticated,
    apiCall,
    refreshToken
};
