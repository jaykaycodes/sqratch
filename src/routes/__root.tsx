import { QueryClientProvider } from '@tanstack/react-query'
import { createRootRoute, Outlet } from '@tanstack/react-router'

import TitleBar from '#/components/title-bar'
import queryClient from '#/lib/queries'
import UIProvider from '#/providers/ui'

// biome-ignore lint/suspicious/noEmptyInterface: <explanation>
export interface RootRouteContext {}

export const Route = createRootRoute({
	component: RootLayout,
})

export function RootLayout() {
	return (
		<QueryClientProvider client={queryClient}>
			<UIProvider>
				<div className="flex h-screen flex-col">
					<TitleBar />
					<main className="h-(--main-height)">
						<Outlet />
					</main>
				</div>
			</UIProvider>
		</QueryClientProvider>
	)
}
