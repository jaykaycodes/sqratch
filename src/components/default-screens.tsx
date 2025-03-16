import * as Icons from 'lucide-react'

import { AppLink } from '#/components/app-link'
export function DefaultNotFound() {
  return (
    <div className="flex h-screen w-screen flex-col items-center justify-center pb-12">
      <h1 className="mb-4 text-2xl font-medium">Page not found</h1>

      <AppLink variant="default" to="/" size="sm">
        Go to Home
      </AppLink>
    </div>
  )
}

export function DefaultPending() {
  return (
    <div className="flex h-full w-full flex-1 self-center flex-col items-center justify-center gap-4">
      <Icons.Loader2 className="h-4 w-4 animate-spin" />
      <div className="text-sm text-muted-foreground">Loading...</div>
    </div>
  )
}
