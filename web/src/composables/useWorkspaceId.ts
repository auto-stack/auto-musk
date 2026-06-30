import { ref } from 'vue'
// Mutable singleton, readable by useAuth without a circular import.
export const currentWorkspaceId = ref<string | null>(null)
