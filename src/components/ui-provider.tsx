import React from 'react'

import { useAtom } from 'jotai'
import { useHotkeys } from 'react-hotkeys-hook'

import { devModeAtom } from '#/stores'

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
	const [isDev, setIsDev] = useAtom(devModeAtom)
	useHotkeys(
		'shift+meta+.',
		() => {
			setIsDev(!isDev)
		},
		{ preventDefault: true },
		[isDev],
	)

	return (
		<TooltipProvider>
			{isDev && (
				<React.Suspense>
					<TanStackRouterDevtools />
				</React.Suspense>
			)}

			<SidebarProvider>{children}</SidebarProvider>
			<Toaster />
		</TooltipProvider>
	)
}
