import './main.css'

import React from 'react'

import {
	createRouter,
	ErrorComponent,
	parseSearchWith,
	RouterProvider,
	stringifySearchWith,
} from '@tanstack/react-router'
import { getDefaultStore } from 'jotai'
import * as jsurl from 'jsurl2'
import ReactDOM from 'react-dom/client'

import { DefaultNotFound, DefaultPending } from './components/default-screens'
import { getQueryClient } from './lib/query-client'
import type { RootRouteContext } from './routes/__root'
import { routeTree } from './routeTree.gen'

const store = getDefaultStore()
const queryClient = getQueryClient()

const defaultContext: RootRouteContext = {
	store,
	queryClient,
}

const router = createRouter({
	routeTree,
	context: defaultContext,
	trailingSlash: 'never',
	defaultPreload: 'intent',
	// Since we're using React Query, we don't want loader calls to ever be stale
	// This will ensure that the loader is always called when the route is preloaded or visited
	defaultPreloadStaleTime: 0,
	parseSearch: parseSearchWith(jsurl.parse),
	stringifySearch: stringifySearchWith(jsurl.stringify),
	defaultNotFoundComponent: DefaultNotFound,
	defaultPendingComponent: DefaultPending,
	defaultErrorComponent: ErrorComponent,
})

declare module '@tanstack/react-router' {
	interface Register {
		router: typeof router
	}
}

const rootElement = document.getElementById('root') as HTMLElement
if (!rootElement.innerHTML) {
	const root = ReactDOM.createRoot(rootElement)
	root.render(
		<React.StrictMode>
			<RouterProvider router={router} />
		</React.StrictMode>,
	)
}
