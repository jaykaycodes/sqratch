import { createRootRoute, Outlet } from '@tanstack/react-router'
import type { getDefaultStore } from 'jotai'

import { UIProvider } from '#/components/ui-provider'

export interface RootRouteContext {
  store: ReturnType<typeof getDefaultStore>
}

export const Route = createRootRoute({
  component: RootLayout,
})

export function RootLayout() {
  return (
    <UIProvider>
      <Outlet />
    </UIProvider>
  )
}
