import React from 'react'

import { sep } from '@tauri-apps/api/path'
import { readDir } from '@tauri-apps/plugin-fs'

import { observable, when } from '@legendapp/state'
import { useMount } from '@legendapp/state/react'
import { synced } from '@legendapp/state/sync'

import Icons from '#/components/icons'
import { TitleBarAction, titleBarActions$ } from '#/components/title-bar'
import { cn, taurpc } from '#/lib/utils'
import { WORKBENCH_TABS, type WorkbenchItem, type WorkbenchTab } from '#/routes/-project/workbench'

export type ConnectionStatus = 'connected' | 'disconnected' | 'loading'

const createProjectStore = (connectionString: string) => {
	let name = connectionString.split('/').pop() ?? ''
	try {
		const url = new URL(connectionString)
		name = url.hostname.split(':')[0]
	} catch (error) {
		console.error(error)
	}

	const connectionStatus$ = observable('loading' as ConnectionStatus)

	const queries$ = synced({
		activate: 'auto',
		get: async (): Promise<WorkbenchItem[]> => {
			const entries = await readDir('.sqratch/queries', { baseDir: import.meta.env.PROJECT_ROOT })
			console.log(entries)
			return entries.map((entry) => {
				const fullPath = [import.meta.env.PROJECT_ROOT, entry.name].join(sep())

				return {
					id: `file://${fullPath}`,
					name: entry.name,
					type: entry.isDirectory ? 'Folder' : 'File',
					path: entry.name.split(sep()).filter(Boolean),
				}
			})
		},
	})

	const entities$ = synced({
		activate: 'auto',
		get: async () => {
			await when(() => connectionStatus$.get() === 'connected')
			return taurpc.db.get_all_entities()
		},
	})

	const store$ = observable({
		name,
		connectionString,
		connectionStatus: connectionStatus$,
		queries: queries$,
		entities: entities$,
		favorites: new Set<string>(),
		ui: {
			workbenchTab: WORKBENCH_TABS[0].label as WorkbenchTab,
			detailsPanel: {
				open: false,
			},
		},
		async connect(connect = true) {
			const fn = connect ? taurpc.db.connect : taurpc.db.disconnect

			if (store$.connectionStatus.get() === 'disconnected') store$.connectionStatus.set('loading')

			try {
				await fn()
				store$.connectionStatus.set(connect ? 'connected' : 'disconnected')
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

export default function ProjectProvider({
	children,
	connectionString,
}: {
	children: React.ReactNode
	connectionString: string
}) {
	const store$ = React.useMemo(() => createProjectStore(connectionString), [connectionString])

	useMount(() => {
		store$.connect().then(() => store$.entities.get())

		const action = observable<TitleBarAction>(() => ({
			tooltip: 'Toggle Details Panel',
			Icon: Icons.PanelRight,
			iconClassName: cn(
				store$.ui.detailsPanel.open.get() ? 'text-foreground' : 'text-muted-foreground',
			),
			onClick: store$.ui.detailsPanel.open.toggle,
		}))
		titleBarActions$.set('details-panel', action)

		return () => {
			store$.connect(false)
			titleBarActions$.delete('details-panel')
		}
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
