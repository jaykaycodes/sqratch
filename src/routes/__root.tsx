import { QueryClientProvider } from '@tanstack/react-query'
import { createRootRoute, Outlet } from '@tanstack/react-router'

import DevTools from '#/components/dev-tools'
import TitleBar from '#/components/title-bar'
import Toaster from '#/components/ui/toaster'
import { queryClient } from '#/lib/utils'

// biome-ignore lint/suspicious/noEmptyInterface: stub
export interface RootRouteContext {}

export const Route = createRootRoute({
	component: RootLayout,
})

function RootLayout() {
	return (
		<QueryClientProvider client={queryClient}>
			<div className="flex h-screen w-screen flex-col overflow-hidden">
				<TitleBar />
				<main className="h-(--main-height) w-full">
					<Outlet />
				</main>
			</div>

			<DevTools />
			<Toaster />
		</QueryClientProvider>
	)
}
