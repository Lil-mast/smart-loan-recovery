/**
 * API origin for the static UI.
 * On Vercel there is no Rust process, so /users must go to the backend host.
 * Override: localStorage.LENDWISE_API_BASE or window.LENDWISE_DEFAULT_API
 */
window.LENDWISE_DEFAULT_API = window.LENDWISE_DEFAULT_API || 'https://lendwise-recovery.fly.dev';

function lendwiseApiBase() {
  const stored = localStorage.getItem('LENDWISE_API_BASE');
  if (stored && stored.trim()) {
    return stored.replace(/\/$/, '');
  }

  const host = window.location.hostname || '';
  const path = window.location.pathname || '';
  const port = window.location.port || '';

  if (host === 'localhost' || host === '127.0.0.1') {
    if (port === '3000' || path === '/app' || path.startsWith('/app/')) {
      return '';
    }
    return 'http://127.0.0.1:3000';
  }

  if (host.endsWith('vercel.app') || host.endsWith('netlify.app')) {
    return String(window.LENDWISE_DEFAULT_API).replace(/\/$/, '');
  }

  if (path === '/app' || path.startsWith('/app/')) {
    return '';
  }

  return String(window.LENDWISE_DEFAULT_API).replace(/\/$/, '');
}
