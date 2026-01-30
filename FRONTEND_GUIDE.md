# Frontend Guide: Smart Loan Recovery System

This guide explains how to access and use the frontend interface for the Smart Loan Recovery System.

## Overview

The frontend is a single-page HTML application located in the `frontend/` directory. It provides a user-friendly web interface to interact with the loan recovery API.

## Accessing the Frontend

### Method 1: Direct File Access
You can open the frontend directly in your browser:

1. **Navigate to the frontend directory**:
   ```
   cd c:\Users\admin\desktop\smart-loan-recovery\frontend
   ```

2. **Open the HTML file**:
   - Double-click on `index.html` in Windows Explorer
   - Or open it in your browser: `file:///c:/Users/admin/desktop/smart-loan-recovery/frontend/index.html`

### Method 2: Through the Running Server (Recommended)
When you run the backend server, the frontend is served as static files:

1. **Start the backend server**:
   ```bash
   cd c:\Users\admin\desktop\smart-loan-recovery
   cargo run
   ```

2. **Access the frontend in your browser**:
   ```
   http://localhost:3000
   ```

### Method 3: Using a Simple HTTP Server
For better development experience, use a local HTTP server:

1. **Using Python** (if installed):
   ```bash
   cd c:\Users\admin\desktop\smart-loan-recovery\frontend
   python -m http.server 8080
   ```
   Then access: `http://localhost:8080`

2. **Using Node.js serve** (if installed):
   ```bash
   cd c:\Users\admin\desktop\smart-loan-recovery\frontend
   npx serve .
   ```
   Then access: `http://localhost:3000` (or the port shown)

## Frontend Features

The frontend provides the following functionality:

### 1. User Management
- **User Registration**: Create new users (borrowers/lenders)
- **User Login**: Authenticate users
- **User List**: View all registered users

### 2. Loan Management
- **Create Loans**: Lenders can create new loans for borrowers
- **View Loans**: Browse all loans in the system
- **Loan Status**: Track loan status (Active, Overdue, Defaulted, Repaid)

### 3. Recovery Features
- **Flag Overdues**: Mark overdue loans for recovery
- **Risk Assessment**: View risk scores for loans
- **Recovery Recommendations**: Get AI-powered recovery suggestions

### 4. Dashboard
- **Overview**: System statistics and summary
- **Recent Activity**: Latest transactions and updates
- **Analytics**: Visual representations of loan data

## API Integration

The frontend communicates with the backend API using the following endpoints:

### Authentication Endpoints
- `POST /auth/login` - User login
- `POST /auth/logout` - User logout
- `GET /auth/me` - Get current user info

### User Management
- `GET /users` - List all users
- `POST /users` - Register new user

### Loan Management
- `GET /loans` - List all loans
- `POST /loans` - Create new loan
- `POST /overdues` - Flag overdue loans
- `POST /recommend/{loan_id}` - Get recovery recommendation

## Frontend Architecture

### HTML Structure
- **Semantic HTML5**: Uses modern HTML5 elements
- **Responsive Design**: Mobile-friendly layout
- **Accessibility**: ARIA labels and semantic markup

### CSS Styling
- **Modern CSS**: Flexbox and Grid layouts
- **Gradient Design**: Professional appearance
- **Interactive Elements**: Hover states and transitions
- **Responsive Breakpoints**: Works on all screen sizes

### JavaScript Functionality
- **Fetch API**: For API communication
- **Async/Await**: Modern JavaScript patterns
- **Error Handling**: User-friendly error messages
- **Local Storage**: Session management

## Configuration

### API Base URL
The frontend expects the API to be available at `http://localhost:3000` by default. This can be modified in the JavaScript section of the HTML file.

### CORS Configuration
The backend is configured to allow requests from the frontend. If you encounter CORS issues, ensure the backend is running with proper CORS settings.

## Development Workflow

### 1. Backend Development
```bash
# Start the backend server
cd c:\Users\admin\desktop\smart-loan-recovery
cargo run

# The server will start on http://localhost:3000
# Frontend will be served from /frontend directory
```

### 2. Frontend Development
```bash
# Option 1: Direct file editing
# Edit frontend/index.html directly

# Option 2: Live reload with HTTP server
cd c:\Users\admin\desktop\smart-loan-recovery\frontend
python -m http.server 8080 --bind 127.0.0.1
```

### 3. Testing
1. Open the frontend in your browser
2. Test user registration and login
3. Create and manage loans
4. Test recovery features

## Troubleshooting

### Common Issues

#### 1. "Cannot connect to API"
- **Solution**: Ensure the backend server is running on `http://localhost:3000`
- **Check**: Run `cargo run` in the project root

#### 2. "CORS errors"
- **Solution**: The backend should handle CORS automatically
- **Check**: Verify the server is running with proper middleware

#### 3. "404 Not Found"
- **Solution**: Ensure you're accessing the correct URL
- **Check**: Use `http://localhost:3000` when server is running

#### 4. "Database errors"
- **Solution**: Check if `loans.db` exists and is accessible
- **Check**: Ensure the database file is not corrupted

### Browser Console Debugging
1. Open Developer Tools (F12)
2. Check Console tab for JavaScript errors
3. Check Network tab for API request failures
4. Verify API responses in the Network tab

## Security Considerations

### Frontend Security
- **Input Validation**: Client-side validation for user inputs
- **XSS Prevention**: Proper escaping of user content
- **HTTPS**: Use HTTPS in production

### API Security
- **Authentication**: Session-based authentication
- **Authorization**: Role-based access control
- **Input Validation**: Server-side validation

## Production Deployment

### Static File Serving
In production, the frontend should be served by a proper web server:

```nginx
server {
    listen 80;
    server_name your-domain.com;
    
    location / {
        root /path/to/frontend;
        try_files $uri $uri/ /index.html;
    }
    
    location /api/ {
        proxy_pass http://localhost:3000/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

### Docker Deployment
The frontend is included in the Docker container and served automatically.

## Customization

### Styling
- Modify the CSS section in `index.html`
- Update color schemes and layouts
- Add responsive breakpoints as needed

### Functionality
- Extend JavaScript functions
- Add new API endpoints
- Implement additional features

## Support

For issues with the frontend:
1. Check the browser console for errors
2. Verify the backend API is running
3. Review the network requests in developer tools
4. Consult the main documentation at `DEVELOPMENT_CHALLENGES.md`

## Future Enhancements

Planned frontend improvements:
- [ ] Real-time updates with WebSockets
- [ ] Advanced charts and analytics
- [ ] Mobile app version
- [ ] Progressive Web App (PWA) features
- [ ] Multi-language support
- [ ] Dark mode theme
