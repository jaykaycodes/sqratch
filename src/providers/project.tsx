import React from 'react'

import { observable, when } from '@legendapp/state'
import { useMount } from '@legendapp/state/react'

import type { SchemaEntity, SchemaInfo } from '#/lib/taurpc'
import { taurpc } from '#/lib/utils'
import type {
	WorkbenchItem,
	WorkbenchItemOrGroup,
	WorkbenchQueryItems,
} from '#/routes/-project/workbench'

type ConnectionStatus = 'connected' | 'disconnected' | 'loading'

const createProjectStore = (connectionString: string) => {
	let name = connectionString.split('/').pop()
	try {
		const url = new URL(connectionString)
		name = url.hostname.split(':')[0]
	} catch (error) {
		console.error(error)
	}

	const store$ = observable({
		name,
		connectionStatus: 'loading' as ConnectionStatus,
		connectionString,
		favorites,
		queries,
		workbench: {
			favorites: {
				open: true,
				heightPct: 40,
				order: 0,
				items: async () => favorites,
			},
			queries: {
				open: true,
				heightPct: 30,
				order: 2,
				items: async () => queries,
			},
			entities: {
				open: true,
				heightPct: 30,
				order: 1,
				items: async () => mapSchemaToWorkbenchItems(store$.schemas.get(), true),
			},
		},
		schemas: async () => {
			await when(() => store$.connectionStatus.get() === 'connected')
			return taurpc.db.get_all_schemas()
		},
		async connect(disconnect = false) {
			const fn = disconnect ? taurpc.db.disconnect : taurpc.db.connect

			if (store$.connectionStatus.get() === 'disconnected') store$.connectionStatus.set('loading')

			try {
				await fn()
				store$.connectionStatus.set(disconnect ? 'disconnected' : 'connected')
			} catch (error) {
				console.error(error)
				store$.connectionStatus.set('disconnected')
			}
		},
	})

	return store$
}

export type ProjectStore = ReturnType<typeof createProjectStore>

// biome-ignore lint/style/noNonNullAssertion: <explanation>
const ProjectContext = React.createContext<ReturnType<typeof createProjectStore>>(null!)

export const ProjectProvider = ({
	children,
	connectionString,
}: {
	children: React.ReactNode
	connectionString: string
}) => {
	const store$ = React.useMemo(() => createProjectStore(connectionString), [connectionString])

	useMount(() => {
		store$.connect()
	})

	return <ProjectContext.Provider value={store$}>{children}</ProjectContext.Provider>
}

export function useProjectStore$() {
	const store$ = React.useContext(ProjectContext)
	if (!store$) {
		throw new Error('useProjectStore$ must be used within a <ProjectProvider />')
	}

	return store$
}

const queries: WorkbenchQueryItems = [
	{
		id: 'dir1',
		type: 'Folder',
		name: 'Reports',
		items: [
			{ id: 'q1', type: 'UserQuery', name: 'Monthly active users' },
			{ id: 'q2', type: 'UserQuery', name: 'Revenue report' },
			{
				id: 'subdir1',
				type: 'Folder',
				name: 'Marketing',
				items: [{ id: 'q3', type: 'UserQuery', name: 'Campaign performance' }],
			},
		],
	},
	{
		id: 'dir2',
		type: 'Folder',
		name: 'Analytics',
		items: [
			{ id: 'q4', type: 'UserQuery', name: 'User retention' },
			{ id: 'q5', type: 'UserQuery', name: 'Funnel analysis' },
		],
	},
]

const favorites: WorkbenchItem[] = [
	{ id: 'fav1', type: 'UserQuery', name: 'Monthly active users' },
	{ id: 'fav2', type: 'Table', name: 'users' },
	{ id: 'fav3', type: 'UserQuery', name: 'Revenue report' },
]

function mapSchemaToWorkbenchItems(schemas: SchemaInfo[], reverse = false) {
	// Create a new array for our result
	let result: WorkbenchItemOrGroup[] = []

	// Check if we have schemas
	if (!schemas.length) return []

	const sortFn = (a: { name: string }, b: { name: string }) => {
		const comparison = a.name.localeCompare(b.name)
		return reverse ? -comparison : comparison
	}

	const toSortedEntities = (entities: SchemaEntity[]) =>
		entities.sort(sortFn).map((entity) => ({
			id: entity.id,
			name: entity.name,
			type: entity.entity_type,
		}))

	// Process each schema
	schemas.sort(sortFn).forEach((schema) => {
		// For public schema, add its items directly to the result
		if (schema.name === 'public') {
			result = [...toSortedEntities(schema.entities), ...result]
		} else {
			// For other schemas, keep them as groups
			result.push({
				id: schema.id,
				name: schema.name,
				type: 'Schema',
				items: toSortedEntities(schema.entities),
			})
		}
	})

	// Sort the top-level schemas (but not the flattened public items)
	return result
}
