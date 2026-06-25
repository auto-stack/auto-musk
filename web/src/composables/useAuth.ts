import { ref } from 'vue'

const TOKEN_KEY = 'musk_jwt'
const token = ref<string | null>(localStorage.getItem(TOKEN_KEY))
const username = ref<string | null>(null)

/** fetch wrapper that injects the JWT bearer token into /api/ requests. */
export async function apiFetch(url: string, init: RequestInit = {}): Promise<Response> {
  const headers = new Headers(init.headers)
  if (token.value) headers.set('Authorization', `Bearer ${token.value}`)
  if (init.body && !headers.has('Content-Type')) headers.set('Content-Type', 'application/json')
  const resp = await fetch(url, { ...init, headers })
  if (resp.status === 401) {
    // token expired/invalid — clear + force re-login
    logout()
  }
  return resp
}

export async function login(user: string, pass: string): Promise<boolean> {
  const resp = await fetch('/api/auth/login', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ username: user, password: pass }),
  })
  if (!resp.ok) return false
  const data = await resp.json()
  token.value = data.token
  localStorage.setItem(TOKEN_KEY, data.token)
  await fetchMe()
  return true
}

export function logout() {
  token.value = null
  username.value = null
  localStorage.removeItem(TOKEN_KEY)
}

export async function fetchMe(): Promise<boolean> {
  if (!token.value) return false
  const resp = await apiFetch('/api/auth/me')
  if (!resp.ok) {
    logout()
    return false
  }
  const data = await resp.json()
  username.value = data.username || null
  return true
}

export function useAuth() {
  return { token, username, login, logout, fetchMe }
}
