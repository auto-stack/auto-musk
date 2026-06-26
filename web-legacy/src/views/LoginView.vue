<script setup lang="ts">
import { ref } from 'vue'
import { useAuth } from '../composables/useAuth'

const { login } = useAuth()
const user = ref('admin')
const pass = ref('admin')
const error = ref('')
const busy = ref(false)

async function submit() {
  busy.value = true
  error.value = ''
  const ok = await login(user.value, pass.value)
  if (!ok) error.value = 'Invalid credentials'
  busy.value = false
}
</script>

<template>
  <div class="login-wrap">
    <form class="login-card" @submit.prevent="submit">
      <div class="logo">🦌</div>
      <h1>Auto Musk</h1>
      <p class="sub">Sign in to your AI coding agent</p>
      <input v-model="user" type="text" placeholder="Username" autocomplete="username" />
      <input v-model="pass" type="password" placeholder="Password" autocomplete="current-password" />
      <div v-if="error" class="error">{{ error }}</div>
      <button type="submit" class="btn-primary" :disabled="busy">{{ busy ? 'Signing in…' : 'Sign in' }}</button>
      <p class="hint">Default: admin / admin</p>
    </form>
  </div>
</template>

<style scoped>
.login-wrap { display: flex; align-items: center; justify-content: center; height: 100vh; }
.login-card {
  background: var(--bg-panel); border: 1px solid var(--border); border-radius: var(--radius);
  padding: 32px; width: 320px; display: flex; flex-direction: column; gap: 12px; align-items: center;
}
.logo { font-size: 40px; }
h1 { font-size: 22px; font-weight: 700; }
.sub { font-size: 13px; color: var(--text-muted); margin-bottom: 8px; }
input {
  width: 100%; padding: 10px 12px; background: var(--bg-input); border: 1px solid var(--border);
  border-radius: var(--radius-sm); color: var(--text-primary); font-size: 14px; outline: none;
  transition: border-color .15s;
}
input:focus { border-color: var(--accent); }
.btn-primary {
  width: 100%; padding: 10px; background: var(--accent); color: var(--accent-foreground);
  border-radius: var(--radius-sm); font-weight: 600; font-size: 14px; transition: background .15s;
}
.btn-primary:hover:not(:disabled) { background: var(--accent-hover); }
.btn-primary:disabled { opacity: .6; }
.error { color: var(--danger); font-size: 12px; }
.hint { font-size: 11px; color: var(--text-muted); margin-top: 4px; }
</style>
