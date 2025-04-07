import React from 'react'

import { observable } from '@legendapp/state'
import { enableReactComponents } from '@legendapp/state/config/enableReactComponents'
import { enableReactTracking } from '@legendapp/state/config/enableReactTracking'
import { Show } from '@legendapp/state/react'
import { useHotkeys } from 'react-hotkeys-hook'

import { SidebarProvider } from '#/components/ui/sidebar'
import { Toaster } from '#/components/ui/sonner'
import { TooltipProvider } from '#/components/ui/tooltip'

enableReactComponents()

// Enable React tracking for debugging
enableReactTracking({
	warnMissingUse: true,
	warnUnobserved: true,
})

const TanStackRouterDevtools = import.meta.env.PROD
	? () => null
	: React.lazy(() =>
			import('@tanstack/router-devtools').then((res) => ({
				default: res.TanStackRouterDevtools,
			})),
		)

export const ui$ = observable({
	devMode: false,
})

const UIProvider = ({ children }: { children: React.ReactNode }) => {
	useHotkeys(
		'shift+meta+.',
		() => ui$.devMode.set(!ui$.devMode.get()),
		{ preventDefault: true },
		[],
	)

	return (
		<TooltipProvider>
			<Show if={ui$.devMode} wrap={React.Suspense}>
				<TanStackRouterDevtools />
			</Show>

			<SidebarProvider>{children}</SidebarProvider>
			<Toaster />
		</TooltipProvider>
	)
}

export default UIProvider
