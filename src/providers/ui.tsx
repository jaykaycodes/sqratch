import React from 'react'

import { observable } from '@legendapp/state'
import { enableReactTracking } from '@legendapp/state/config/enableReactTracking'
import { Show } from '@legendapp/state/react'
import { ThemeProvider } from 'next-themes'
import { useHotkeys } from 'react-hotkeys-hook'

import { Toaster } from '#/components/ui/sonner'
import { TooltipProvider } from '#/components/ui/tooltip'

// Enable React tracking for debugging
enableReactTracking({
	warnMissingUse: true,
	warnUnobserved: true,
})

const TanStackRouterDevtools = import.meta.env.PROD
	? () => null
	: React.lazy(() =>
			import('@tanstack/react-router-devtools').then((res) => ({
				default: res.TanStackRouterDevtools,
			})),
		)

const TanStackQueryDevtools = import.meta.env.PROD
	? () => null
	: React.lazy(() =>
			import('@tanstack/react-query-devtools').then((res) => ({
				default: res.ReactQueryDevtools,
			})),
		)

export const ui$ = observable({
	devMode: false,
	windowActions: null as React.ReactNode | null,
})

const UIProvider = ({ children }: { children: React.ReactNode }) => {
	useHotkeys(
		'shift+meta+.',
		() => ui$.devMode.set(!ui$.devMode.get()),
		{ preventDefault: true },
		[],
	)

	return (
		<ThemeProvider attribute="class">
			<TooltipProvider>
				<Show if={ui$.devMode} wrap={React.Suspense}>
					<TanStackRouterDevtools />
					<TanStackQueryDevtools />
				</Show>

				{children}

				<Toaster />
			</TooltipProvider>
		</ThemeProvider>
	)
}

export default UIProvider
