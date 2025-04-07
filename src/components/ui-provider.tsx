import React from 'react'

import { Show } from '@legendapp/state/react'
import { useHotkeys } from 'react-hotkeys-hook'

import store$ from '#/store'

import { SidebarProvider } from './ui/sidebar'
import { Toaster } from './ui/sonner'
import { TooltipProvider } from './ui/tooltip'

const TanStackRouterDevtools = import.meta.env.PROD
	? () => null
	: React.lazy(() =>
			import('@tanstack/router-devtools').then((res) => ({
				default: res.TanStackRouterDevtools,
			})),
		)

export const UIProvider = ({ children }: { children: React.ReactNode }) => {
	useHotkeys(
		'shift+meta+.',
		() => store$.devMode.set(!store$.devMode.get()),
		{ preventDefault: true },
		[],
	)

	return (
		<TooltipProvider>
			<Show if={store$.devMode} wrap={React.Suspense}>
				<TanStackRouterDevtools />
			</Show>

			<SidebarProvider>{children}</SidebarProvider>
			<Toaster />
		</TooltipProvider>
	)
}
