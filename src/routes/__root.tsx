import { createRootRoute, Outlet } from '@tanstack/react-router'

import UIProvider from '#/providers/ui'

// biome-ignore lint/suspicious/noEmptyInterface: <explanation>
export interface RootRouteContext {}

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
