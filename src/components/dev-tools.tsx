import React from 'react'

import { enableReactTracking } from '@legendapp/state/config/enableReactTracking'
import { Show } from '@legendapp/state/react'
import { useHotkeys } from 'react-hotkeys-hook'

import GlobalStore$ from '#/stores/global-store'

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

export default function DevTools() {
	useHotkeys('shift+meta+.', () => GlobalStore$.devMode.toggle(), { preventDefault: true }, [])

	return (
		<Show if={GlobalStore$.devMode} wrap={React.Suspense}>
			<TanStackRouterDevtools />
			<TanStackQueryDevtools />
		</Show>
	)
}
