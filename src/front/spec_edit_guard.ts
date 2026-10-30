// The editor owns the draft; the app shell must not unmount it while dirty.
let dirty = false
let installed = false
export function markSpecDirty(value: boolean): void {
  dirty = value
  if (!installed) {
    installed = true
    window.addEventListener('beforeunload', (event) => {
      if (dirty) { event.preventDefault(); event.returnValue = '' }
    })
  }
}
export function specMayLeave(): boolean { return !dirty }
